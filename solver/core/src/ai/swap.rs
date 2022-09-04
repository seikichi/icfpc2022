use log::info;

use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct SwapAI {}

impl HeadAI for SwapAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> isl::Program {
        // let mut state = initial_state.clone();
        let blocks = initial_state.blocks.iter().collect::<Vec<_>>();

        let colors = blocks.iter().map(|b| b.1.color).collect::<Vec<_>>();
        let mut similarity = vec![vec![0; colors.len()]; blocks.len()]; // ブロックiを色jで塗ったときのsimilarity

        for i in 0..blocks.len() {
            for j in 0..colors.len() {
                let block = blocks[i].1;
                similarity[i][j] = simulator::calc_partial_one_color_similarity(
                    block.p, block.size, colors[j], image,
                )
            }
        }

        let mut color_origin = (0..blocks.len()).collect::<Vec<_>>(); // color_origin[i]は、現在のi番目の位置にいるブロックの色が、初期状態で何番目のブロックの色だったかを表す

        let mut program = vec![];
        let mut updated = true;
        let mut iter_count = 0;
        while updated && iter_count < 10000 {
            info!("iter_count {iter_count}");
            iter_count += 1;
            updated = false;
            let mut min_cost_delta = std::i64::MAX;
            let mut best_pair = (0, 0);
            for i in 0..blocks.len() {
                for j in (i + 1)..blocks.len() {
                    if blocks[i].1.size == blocks[j].1.size {
                        let (bid_i, b_i) = blocks[i];
                        let (bid_j, _) = blocks[j];
                        let swap_move = isl::Move::Swap {
                            a: bid_i.clone(),
                            b: bid_j.clone(),
                        };
                        let move_cost = simulator::move_cost_without_state(
                            &swap_move,
                            b_i.area() as usize,
                            image.width(),
                            image.height(),
                            initial_state.cost_coeff_version,
                        );
                        let sim_before =
                            similarity[i][color_origin[i]] + similarity[j][color_origin[j]];
                        let sim_after =
                            similarity[i][color_origin[j]] + similarity[j][color_origin[i]];

                        let cost_delta = sim_after + move_cost - sim_before;
                        if cost_delta < min_cost_delta {
                            min_cost_delta = cost_delta;
                            best_pair = (i, j);
                        }
                    }
                }
            }
            // swap
            if min_cost_delta < 0 {
                updated = true;
                let (i, j) = best_pair;
                info!("min_cost_delta {min_cost_delta}, {i} {j}");
                let swap_move = isl::Move::Swap {
                    a: blocks[i].0.clone(),
                    b: blocks[j].0.clone(),
                };
                color_origin.swap(i, j);
                program.push(swap_move);
            }
        }
        info!("end swap ai");
        isl::Program(program)
    }
}
