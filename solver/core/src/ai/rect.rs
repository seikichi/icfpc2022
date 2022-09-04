use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

use std::collections::HashMap;
use std::thread::current;

pub struct RectAI {}

impl HeadAI for RectAI {
    fn solve(&mut self, image: &image::Image, _initial_state: &simulator::State) -> isl::Program {
        let height = image.height();
        let width = image.width();

        let mut current_image = image.clone();
        let mut program = vec![];

        // とりあえず一番多い色で塗っとく
        let mut counter = HashMap::new();
        for row in &image.0 {
            for color in row {
                let icolor = (*color * 255.0).round().as_ivec4();
                *counter.entry(icolor).or_insert(0) += 1;
            }
        }
        let mut max_freq = 0;
        let mut color = isl::Color::ZERO;
        for (k, &v) in counter.iter() {
            if max_freq < v {
                max_freq = v;
                color = k.as_vec4() / 255.0;
            }
        }
        program.push(isl::Move::Color {
            block_id: isl::BlockId::new(&vec![0]),
            color,
        });
        println!("first: {:?}", color);
        for i in 0..height {
            for j in 0..width {
                current_image.0[i][j] = color;
            }
        }

        let mut id = 0;
        for _ in 0..20 {
            // 一番違ってそうなやつ探す
            let mut counter = HashMap::new();
            for i in 0..height {
                for j in 0..width {
                    if image.0[i][j] != current_image.0[i][j] {
                        let color = image.0[i][j];
                        let icolor = (color * 255.0).round().as_ivec4();
                        *counter.entry(icolor).or_insert(0) += 1;
                    }
                }
            }
            let mut max_freq = 0;
            let mut color = isl::Color::ZERO;
            for (k, &v) in counter.iter() {
                if max_freq < v {
                    max_freq = v;
                    color = k.as_vec4() / 255.0;
                }
            }
            println!("next: {:?}", color);
            // 一番違ってそうなやつを塗り潰す座標決める
            let mut min = isl::Point::new(width as i32, height as i32);
            let mut max = isl::Point::ZERO;
            for i in 0..height {
                for j in 0..width {
                    if image.0[i][j] == color {
                        min.x = std::cmp::min(min.x, j as i32);
                        min.y = std::cmp::min(min.y, i as i32);
                        max.x = std::cmp::max(max.x, j as i32);
                        max.y = std::cmp::max(max.y, i as i32);
                    }
                }
            }
            if min.x == max.x || min.y == max.y {
                break;
            }
            if min.x == 0 || min.y == 0 {
                break;
            }
            println!("{:?}, {:?}", min, max);
            // 塗る
            for i in min.y..=max.y {
                for j in min.x..=max.x {
                    current_image.0[i as usize][j as usize] = color;
                }
            }
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
                    color,
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 2, 0]),
                    b: isl::BlockId::new(&vec![id, 2, 1]),
                },
                isl::Move::Merge {
                    a: isl::BlockId::new(&vec![id, 2, 2]),
                    b: isl::BlockId::new(&vec![id, 2, 3]),
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

        isl::Program(program)
    }
}
