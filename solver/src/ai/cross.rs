use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use std::collections::HashMap;

pub struct CrossAI {
    pub size: usize,
}

impl CrossAI {
    // image の [min.y, max.y) と [min.x, max.x) をやるぞ！
    // depth が self.size 未満なら打ち切り
    fn draw(
        &self,
        block_id: isl::BlockId,
        image: &image::Image,
        min: isl::Point,
        max: isl::Point,
        depth: usize,
    ) -> Vec<isl::Move> {
        if depth >= self.size {
            // 末端なら
            let mut sum = isl::Color::ZERO;
            for y in min.y..max.y {
                for x in min.x..max.x {
                    sum += image.0[y as usize][x as usize];
                }
            }
            let area = (max - min).y * (max - min).x;
            let color = sum / (area as f32);
            return vec![isl::Move::Color { block_id, color }];
        }

        // p-cut して4領域再帰的に処理
        let ave = (min + max) / 2;
        let mut result = vec![isl::Move::PCut {
            block_id: block_id.clone(),
            point: ave,
        }];

        let next_points = [
            ((min.x, min.y), (ave.x, ave.y)),
            ((ave.x, min.y), (max.x, ave.y)),
            ((ave.x, ave.y), (max.x, max.y)),
            ((min.x, ave.y), (ave.x, max.y)),
        ];
        for (i, ps) in next_points.iter().enumerate() {
            let mut next_block_id = block_id.clone();
            next_block_id.0.push_back(i as u32);
            result.extend(self.draw(
                next_block_id,
                image,
                isl::Point::new(ps.0 .0, ps.0 .1),
                isl::Point::new(ps.1 .0, ps.1 .1),
                depth + 1,
            ));
        }

        // 全て同じ色に塗るなら圧縮可能
        if result.len() == 5 {
            if let (
                isl::Move::Color {
                    block_id: _,
                    color: c1,
                },
                isl::Move::Color {
                    block_id: _,
                    color: c2,
                },
                isl::Move::Color {
                    block_id: _,
                    color: c3,
                },
                isl::Move::Color {
                    block_id: _,
                    color: c4,
                },
            ) = (&result[1], &result[2], &result[3], &result[4])
            {
                if c1 == c2 && c2 == c3 && c3 == c4 {
                    return vec![isl::Move::Color {
                        block_id,
                        color: c1.clone(),
                    }];
                }
            }
        }

        result
    }
}

impl HeadAI for CrossAI {
    fn solve(&mut self, image: &image::Image) -> isl::Program {
        // 再帰的 に pcut してく
        // 各マスの色に何を塗るかを集計して
        // 分割しなくていいならやめる (-> 再帰でなんかそれっぽく書く)
        let height = image.0.len() as i32;
        let width = image.0[0].len() as i32;

        let result = self.draw(
            isl::BlockId::new(&vec![0]),
            image,
            isl::Point::ZERO,
            isl::Point::new(width, height),
            0,
        );

        // 一番多いのは最初にぬっちゃう
        let mut hash = HashMap::new();
        for m in &result {
            if let isl::Move::Color { block_id: _, color } = m {
                let s = isl::format_color(color);
                let counter = hash.entry(s).or_insert(0);
                *counter += 1;
            }
        }
        let mut key = "";
        let mut value = 0 as i32;
        for (k, &v) in hash.iter() {
            if value < v {
                key = k;
                value = v;
            }
        }

        if key != "" {
            for m in &result {
                if let isl::Move::Color { block_id: _, color } = m {
                    if isl::format_color(color) == key {
                        let mut refined = vec![isl::Move::Color {
                            block_id: isl::BlockId::new(&vec![0]),
                            color: color.clone(),
                        }];
                        for n in &result {
                            if let isl::Move::Color {
                                block_id: _,
                                color: ncolor,
                            } = n
                            {
                                if isl::format_color(ncolor) == key {
                                    continue;
                                }
                            }
                            refined.push(n.clone());
                        }
                        return isl::Program(refined);
                    }
                }
            }
        }

        isl::Program(result)
    }
}
