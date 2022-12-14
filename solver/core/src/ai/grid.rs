use smallvec::smallvec;

use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct GridAI {
    pub rows: usize,
    pub cols: usize,
}

impl HeadAI for GridAI {
    fn solve(&mut self, image: &image::Image, _initial_state: &simulator::State) -> isl::Program {
        let height = image.height();
        let width = image.width();

        let mut result = vec![];
        let grid_height = (height / self.rows) as i32;
        let grid_width = (width / self.cols) as i32;

        let mut block_id = smallvec![0];

        // y 軸で切ってく...
        for i in 1..self.rows + 1 {
            // 最後の行は Y 軸のカットをしてはいけない
            if i < self.rows {
                result.push(isl::Move::LCut {
                    block_id: isl::BlockId(block_id.clone()),
                    orientation: isl::Orientation::Horizontal,
                    line_number: grid_height * (i as i32),
                });
            }

            let mut x_block_id = block_id.clone();
            if i < self.rows {
                // block_id を増やすのはカットしたときだけ！
                x_block_id.push(0);
            }
            for j in 1..self.cols + 1 {
                if j < self.cols {
                    // 最後の列は X 軸のカットをしてはいけない
                    result.push(isl::Move::LCut {
                        block_id: isl::BlockId(x_block_id.clone()),
                        orientation: isl::Orientation::Vertical,
                        line_number: grid_width * (j as i32),
                    });
                    x_block_id.push(0);
                }
                let y_from = grid_height * ((i - 1) as i32);
                let y_to = grid_height * (i as i32);
                let x_from = grid_width * ((j - 1) as i32);
                let x_to = grid_width * (j as i32);
                let mut sum = isl::Color::ZERO;
                for y in y_from..y_to {
                    for x in x_from..x_to {
                        sum += image.0[y as usize][x as usize];
                    }
                }
                result.push(isl::Move::Color {
                    block_id: isl::BlockId(x_block_id.clone()),
                    color: sum / (grid_height * grid_width) as f32,
                });

                if j < self.cols {
                    // 最後の列は X 軸のカットをしないので pop 不要
                    x_block_id.pop();
                }
                // 右へ移動
                x_block_id.push(1);
            }
            // 上へ移動
            block_id.push(1);
        }

        isl::Program(result)
    }
}
