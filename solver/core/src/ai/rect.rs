use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct RectAI {}

#[derive(Debug, Clone, PartialEq)]
struct Area {
    min: isl::Point,
    max: isl::Point,
    color: isl::Color,
    size: i32,
}

impl HeadAI for RectAI {
    fn solve(&mut self, image: &image::Image, _initial_state: &simulator::State) -> isl::Program {
        let height = image.height();
        let width = image.width();

        let mut _current_image = image.clone();
        let mut program = vec![];
        let _threashold = 1000;

        let mut visited = vec![vec![false; width]; height];

        let mut areas = vec![];

        for i in 0..height {
            for j in 0..width {
                if visited[i][j] {
                    continue;
                }
                let dyxs = vec![
                    isl::Point::new(1, 0),
                    isl::Point::new(1, 1),
                    isl::Point::new(1, -1),
                    isl::Point::new(-1, 0),
                    isl::Point::new(-1, 1),
                    isl::Point::new(-1, -1),
                    isl::Point::new(0, 1),
                    isl::Point::new(0, -1),
                ];

                let mut size = 0;
                let mut min = isl::Point::new(width as i32, height as i32);
                let mut max = isl::Point::ZERO;
                let color = image.0[i][j];
                let mut sum_color = isl::Color::ZERO;

                let mut que = vec![isl::Point::new(j as i32, i as i32)];
                while !que.is_empty() {
                    let cur = que.pop().unwrap();
                    if visited[cur.y as usize][cur.x as usize] {
                        continue;
                    }

                    size += 1;
                    sum_color += image.0[cur.y as usize][cur.x as usize];
                    visited[cur.y as usize][cur.x as usize] = true;
                    min.x = std::cmp::min(min.x, cur.x as i32);
                    min.y = std::cmp::min(min.y, cur.y as i32);
                    max.x = std::cmp::max(max.x, cur.x as i32);
                    max.y = std::cmp::max(max.y, cur.y as i32);

                    for d in &dyxs {
                        // for c in 1..25 {
                        for c in 1..2 {
                            // TODO: tekito-
                            let next = cur + c * (*d);
                            if 0 <= next.y
                                && next.y < height as i32
                                && 0 <= next.x
                                && next.x < width as i32
                                && (image.0[next.y as usize][next.x as usize] - color).length()
                                    < 0.1
                            {
                                que.push(next);
                            }
                        }
                    }
                }

                areas.push(Area {
                    min,
                    max,
                    color: sum_color / size as f32,
                    size,
                })
            }
        }

        areas.sort_by(|lhs, rhs| rhs.size.cmp(&lhs.size));
        // 一番多い領域の色で塗っとく
        program.push(isl::Move::Color {
            block_id: isl::BlockId::new(&vec![0]),
            color: areas[0].color,
        });

        // 大きい順に塗る
        let mut id = 0;
        for i in 1..20 {
            println!("{:?}", areas[i]);
            let mut min = areas[i].min;
            let mut max = areas[i].max;
            // let color = areas[i].color;

            if min.x == max.x || min.y == max.y {
                continue;
            }
            min.x = std::cmp::max(1, min.x);
            min.y = std::cmp::max(1, min.y);
            max.x = std::cmp::min((width - 1) as i32, max.x);
            max.y = std::cmp::min((height - 1) as i32, max.y);

            let ave_color = image.average(min, max - min);
            program.extend(vec![
                isl::Move::PCut {
                    block_id: isl::BlockId::new(&vec![id]),
                    point: min,
                },
                isl::Move::PCut {
                    block_id: isl::BlockId::new(&vec![id, 2]),
                    point: max,
                },
                isl::Move::Color {
                    block_id: isl::BlockId::new(&vec![id, 2, 0]),
                    // color,
                    color: ave_color,
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 2, 1]),
                    b: isl::BlockId::new(&vec![id, 2, 2]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 2, 3]),
                    b: isl::BlockId::new(&vec![id, 2, 0]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id + 1]),
                    b: isl::BlockId::new(&vec![id + 2]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 0]),
                    b: isl::BlockId::new(&vec![id, 1]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 3]),
                    b: isl::BlockId::new(&vec![id + 3]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id + 4]),
                    b: isl::BlockId::new(&vec![id + 5]),
                },
            ]);
            id += 6
        }

        // // とりあえず一番多い色で塗っとく
        // let mut counter = HashMap::new();
        // for row in &image.0 {
        //     for color in row {
        //         let icolor = (*color * 255.0).round().as_ivec4();
        //         *counter.entry(icolor).or_insert(0) += 1;
        //     }
        // }
        // let mut max_freq = 0;
        // let mut color = isl::Color::ZERO;
        // for (k, &v) in counter.iter() {
        //     if max_freq < v {
        //         max_freq = v;
        //         color = k.as_vec4() / 255.0;
        //     }
        // }
        // program.push(isl::Move::Color {
        //     block_id: isl::BlockId::new(&vec![0]),
        //     color,
        // });
        // println!("first: {:?}", color);
        // for i in 0..height {
        //     for j in 0..width {
        //         current_image.0[i][j] = color;
        //     }
        // }

        // let mut id = 0;
        // for _ in 0..5 {
        //     // 一番違ってそうなやつ探す
        //     let mut counter = HashMap::new();
        //     for i in 0..height {
        //         for j in 0..width {
        //             if image.0[i][j] != current_image.0[i][j] {
        //                 let color = image.0[i][j];
        //                 let icolor = (color * 255.0).round().as_ivec4();
        //                 *counter.entry(icolor).or_insert(0) += 1;
        //             }
        //         }
        //     }
        //     let mut max_freq = 0;
        //     let mut color = isl::Color::ZERO;
        //     for (k, &v) in counter.iter() {
        //         if max_freq < v {
        //             max_freq = v;
        //             color = k.as_vec4() / 255.0;
        //         }
        //     }
        //     println!("next: {:?}", color);
        //     // 一番違ってそうなやつを塗り潰す座標決める
        //     let mut min = isl::Point::new(width as i32, height as i32);
        //     let mut max = isl::Point::ZERO;
        //     for i in 0..height {
        //         for j in 0..width {
        //             if image.0[i][j] == color && current_image.0[i][j] != color {
        //                 min.x = std::cmp::min(min.x, j as i32);
        //                 min.y = std::cmp::min(min.y, i as i32);
        //                 max.x = std::cmp::max(max.x, j as i32);
        //                 max.y = std::cmp::max(max.y, i as i32);
        //             }
        //         }
        //     }
        //     println!("{:?}, {:?}", min, max);
        //     if min.x == max.x || min.y == max.y {
        //         break;
        //     }
        //     if min.x == 0 || min.y == 0 {
        //         break;
        //     }
        //     // 塗る
        //     for i in min.y..=max.y {
        //         for j in min.x..=max.x {
        //             current_image.0[i as usize][j as usize] = color;
        //         }
        //     }
        //     program.extend(vec![
        //         isl::Move::PCut {
        //             block_id: isl::BlockId::new(&vec![id]),
        //             point: min,
        //         },
        //         isl::Move::PCut {
        //             block_id: isl::BlockId::new(&vec![id, 2]),
        //             point: max,
        //         },
        //         isl::Move::Color {
        //             block_id: isl::BlockId::new(&vec![id, 2, 0]),
        //             color,
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id, 2, 0]),
        //             b: isl::BlockId::new(&vec![id, 2, 1]),
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id, 2, 2]),
        //             b: isl::BlockId::new(&vec![id, 2, 3]),
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id + 1]),
        //             b: isl::BlockId::new(&vec![id + 2]),
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id, 0]),
        //             b: isl::BlockId::new(&vec![id, 1]),
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id, 3]),
        //             b: isl::BlockId::new(&vec![id + 3]),
        //         },
        //         isl::Move::Merge {
        //             a: isl::BlockId::new(&vec![id + 4]),
        //             b: isl::BlockId::new(&vec![id + 5]),
        //         },
        //     ]);
        //     id += 6
        // }

        isl::Program(program)
    }
}
