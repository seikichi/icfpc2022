use crate::ai;
use crate::image;
use crate::image::Image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::calc_score;
use crate::simulator::simulate_partial;
use crate::simulator::State;
use log::info;
use rand::Rng;

use super::HeadAI;

pub enum OptimizeAlgorithm {
    HillClimbing,
    Annealing,
}

pub struct RefineAi {
    pub n_iters: usize,
    pub algorithm: OptimizeAlgorithm,
}

impl ai::ChainedAI for RefineAi {
    fn solve(
        &mut self,
        image: &image::Image,
        initial_state: &State,
        initial_program: &Program,
    ) -> Program {
        // TODO seed_from_u64
        let mut rng = rand::thread_rng();

        let mut prev_program = initial_program.clone();
        let mut current_score =
            simulator::calc_score(initial_program, image, initial_state).unwrap();

        let mut best_program = initial_program.clone();
        let mut best_score = current_score;

        let mut temperature;
        let initial_temperature = 1.0;

        for iter in 0..self.n_iters {
            // tweak temperature
            let progress = (iter as f64) / (self.n_iters as f64);
            temperature = initial_temperature * (1.0 - progress) * (-progress).exp2();

            if prev_program.0.len() == 0 {
                break;
            }
            let (candidate_program, description) =
                match self.make_neighbor(&prev_program, image, initial_state, &mut rng) {
                    Some(x) => x,
                    None => continue,
                };
            let new_score = match calc_score(&candidate_program, image, &initial_state) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // 新しい解を受理するか決める
            let accept = match self.algorithm {
                OptimizeAlgorithm::HillClimbing => new_score < current_score,
                OptimizeAlgorithm::Annealing => {
                    if new_score < current_score {
                        true
                    } else {
                        // new_score >= current_score
                        let delta = (new_score - current_score) as f64;
                        let accept_prob = (-delta / temperature).exp();
                        rng.gen::<f64>() < accept_prob
                    }
                }
            };

            if accept {
                info!("iter: {:3}, score: {:7} {}", iter, new_score, description);
                prev_program = candidate_program;
                current_score = new_score;

                if new_score < best_score {
                    best_score = new_score;
                    best_program = prev_program.clone();
                }
            }
        }

        best_program
    }
}

impl RefineAi {
    // 近傍解を作る
    fn make_neighbor(
        &self,
        prev_program: &Program,
        image: &Image,
        initial_state: &State,
        rng: &mut impl Rng,
    ) -> Option<(Program, String)> {
        let mut next_program = prev_program.clone();
        let t = rng.gen_range(0..next_program.0.len());
        let mv = next_program.0[t].clone();
        let mut description = "nop".to_string();
        match mv {
            // TODO swap color timing
            Move::PCut {
                ref block_id,
                point,
            } => {
                let r = rng.gen_range(0..8);
                if r > 0 {
                    // change PCut point
                    let dx = rng.gen_range(-5..=5);
                    let dy = rng.gen_range(-5..=5);
                    let npoint = Point::new(point.x + dx, point.y + dy);
                    next_program.0[t] = Move::PCut {
                        block_id: block_id.clone(),
                        point: npoint,
                    };
                    description = format!("move PCut dx:{} dy:{} {}", dx, dy, next_program.0[t]);
                } else if r == 1 {
                    next_program.0.remove(t);
                    if let Some(result) = Self::remove_all_child(&next_program, block_id) {
                        next_program = result;
                    } else {
                        return None;
                    }
                    next_program = self.solve_by_dp_ai_one_block(
                        next_program,
                        &block_id,
                        image,
                        initial_state,
                        rng,
                    );
                    description = format!("Remove PCut & divide by DpAI");
                }
            }
            Move::LCut {
                ref block_id,
                orientation,
                line_number,
            } => {
                // TODO
                let r = rng.gen_range(0..8);
                if r > 0 {
                    // change LCut position
                    let d = rng.gen_range(-5..=5);
                    next_program.0[t] = Move::LCut {
                        block_id: block_id.clone(),
                        orientation,
                        line_number: line_number + d,
                    };
                    description = format!("move LCut d:{} {}", d, next_program.0[t]);
                } else {
                    next_program.0.remove(t);
                    if let Some(result) = Self::remove_all_child(&next_program, block_id) {
                        next_program = result;
                    } else {
                        return None;
                    }
                    next_program = self.solve_by_dp_ai_one_block(
                        next_program,
                        &block_id,
                        image,
                        initial_state,
                        rng,
                    );
                    description = format!("Remove LCut & divide by DpAI");
                }
            }
            Move::Color {
                ref block_id,
                color: prev_color,
            } => {
                let mut state = initial_state.clone();
                simulate_partial(&mut state, &prev_program.0[0..t]).unwrap();
                let block = state.blocks[block_id];
                let r = rng.gen_range(0..2);
                let color = if r == 0 {
                    // random sampling
                    let x = block.p.x + rng.gen_range(0..block.size.x);
                    let y = block.p.y + rng.gen_range(0..block.size.y);
                    image.0[y as usize][x as usize]
                } else {
                    // average
                    image.average(block.p, block.size)
                };
                if prev_color == color {
                    return None;
                }
                next_program.0[t] = Move::Color {
                    block_id: block_id.clone(),
                    color,
                };
                description = format!("change Color: {}", next_program.0[t]);
            }
            _ => {
                // Do nothing
                return None;
            }
        }
        Some((next_program, description))
    }

    fn solve_by_dp_ai_one_block(
        &self,
        program: Program,
        block_id: &BlockId,
        image: &Image,
        initial_state: &State,
        rng: &mut impl Rng,
    ) -> Program {
        let mut program = program;
        let mut state = initial_state.clone();
        let end_state = simulator::simulate_all(&program, &mut state).unwrap();
        let d = rng.gen_range(2..=4);
        let c = rng.gen_range(3..=6);
        let temp_state = end_state.block_state(block_id.clone());
        let mut dp_ai = ai::DpAI::new(d, c);
        let dp_program = dp_ai.solve(image, &temp_state);
        program.0.append(&mut dp_program.0.clone());
        program.remove_redundant_color_move();
        return program;
    }

    fn remove_all_child(prev_program: &Program, target_block_id: &BlockId) -> Option<Program> {
        let mut next_program = Program(vec![]);
        for i in 0..prev_program.0.len() {
            match prev_program.0[i] {
                Move::PCut {
                    ref block_id,
                    point: _,
                } => {
                    if target_block_id.is_child(block_id) {
                        continue;
                    }
                }
                Move::LCut {
                    ref block_id,
                    orientation: _,
                    line_number: _,
                } => {
                    if target_block_id.is_child(block_id) {
                        continue;
                    }
                }
                Move::Color {
                    ref block_id,
                    color: _,
                } => {
                    if target_block_id.is_child(block_id) {
                        continue;
                    }
                }
                // Swap, Mergeが子に含まれる場合は失敗
                Move::Swap { ref a, ref b } => {
                    if target_block_id.is_child(a) || target_block_id.is_child(b) {
                        return None;
                    }
                }
                Move::Merge { ref a, ref b } => {
                    if target_block_id.is_child(a) || target_block_id.is_child(b) {
                        return None;
                    }
                }
            }
            next_program.0.push(prev_program.0[i].clone());
        }
        return Some(next_program);
    }
}
