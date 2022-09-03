use std::time::{Duration, Instant};

use crate::{
    ai::ChainedAI,
    image::Image,
    isl::{Move, Orientation, Program},
    simulator::{calc_score, simulate_all, ProgramExecError, State},
};
use glam::IVec2;
use rand::prelude::*;

pub struct AnnealingAI {
    pub time_limit: Duration,
}

impl ChainedAI for AnnealingAI {
    fn solve(&mut self, image: &Image, initial_state: &State, initial_program: &Program) -> Program {
        let mut solution = initial_program.clone();
        let mut rng = SmallRng::from_entropy();
        let mut current_score = self.calc_ann_score(&solution, image, initial_state).unwrap();
        let start_at = Instant::now();

        let mut best_solution = solution.clone();
        let mut best_score = current_score;

        let initial_temperature = 100.0;
        let mut temperature = initial_temperature;

        let mut iter = 0;
        loop {
            // check time limit
            iter += 1;
            if iter % 100 == 0 {
                let elapsed = Instant::now() - start_at;
                if elapsed >= self.time_limit {
                    eprintln!("iter = {}", iter);
                    return best_solution;
                }

                // tweak temperature
                let progress = elapsed.as_secs_f64() / self.time_limit.as_secs_f64();
                temperature = initial_temperature * (1.0 - progress) * (-progress).exp2();
            }

            // move to neighbor
            // 方針: PCut か LCut を一つ選び、オフセットをランダムに±1する
            let mut candidates = vec![];
            for (i, mv) in solution.0.iter().enumerate() {
                match mv {
                    Move::LCut { .. } => candidates.push(i),
                    Move::PCut { .. } => candidates.push(i),
                    _ => {}
                }
            }
            if candidates.is_empty() {
                return solution;
            }
            let i_chosen = candidates[rng.gen::<usize>() % candidates.len()];
            let old = solution.0[i_chosen].clone();
            let state = simulate_all(&solution, &initial_state).unwrap();
            let delta = 5; // TODO
            let modified = match old {
                Move::LCut {
                    ref block_id,
                    orientation,
                    line_number,
                } => {
                    let block = state.blocks.get(block_id).unwrap();
                    let offset = line_number
                        - match orientation {
                            Orientation::Horizontal => block.p.y,
                            Orientation::Vertical => block.p.x,
                        };
                    let max_offset = match orientation {
                        Orientation::Horizontal => block.size.y - 1,
                        Orientation::Vertical => block.size.x - 1,
                    };
                    let mut next = None;
                    if offset + delta <= max_offset {
                        next = Some(line_number + delta);
                    }
                    if offset - delta >= 0 && (next.is_none() || rng.gen::<f64>() < 0.5) {
                        next = Some(line_number - delta);
                    }
                    if next.is_none() {
                        // 動かせない
                        continue;
                    }
                    eprintln!(
                        "LCut {} {}: {} -> {}",
                        block_id,
                        orientation,
                        line_number,
                        next.unwrap()
                    );
                    Move::LCut {
                        block_id: block_id.clone(),
                        orientation,
                        line_number: next.unwrap(),
                    }
                }
                Move::PCut {
                    ref block_id,
                    point,
                } => {
                    let block = state.blocks.get(block_id).unwrap();
                    let dx = [-1, -1, -1, 0, 0, 1, 1, 1];
                    let dy = [-1, 0, 1, -1, 1, -1, 0, 1];
                    let mut n_candidates = 0;
                    let mut next = None;
                    for i in 0..8 {
                        let next_p = point + IVec2::new(dx[i], dy[i]) * delta;
                        if next_p.x <= block.p.x
                            || next_p.x >= block.p.x + block.size.x
                            || next_p.y <= block.p.y
                            || next_p.y >= block.p.y + block.size.y
                        {
                            continue;
                        }
                        n_candidates += 1;
                        if rng.gen::<f64>() <= 1.0 / (n_candidates as f64) {
                            next = Some(next_p);
                        }
                    }
                    if next.is_none() {
                        // 動かせない
                        continue;
                    }
                    eprintln!("PCut {}: {} -> {}", block_id, point, next.unwrap());
                    Move::PCut {
                        block_id: block_id.clone(),
                        point: next.unwrap(),
                    }
                }
                _ => unreachable!(),
            };
            solution.0[i_chosen] = modified;

            let new_score = match self.calc_ann_score(&mut solution, image, initial_state) {
                Ok(x) => x,
                Err(_) => {
                    eprintln!("failed to move.. rollback.");
                    solution.0[i_chosen] = old;
                    continue;
                }
            };
            eprintln!("new_score = {new_score}");

            // 新しい解を受理するか決める
            let accept = {
                if new_score < current_score {
                    true
                } else {
                    // new_score >= current_score
                    let delta = new_score - current_score;
                    let accept_prob = (-delta / temperature).exp();
                    rng.gen::<f64>() < accept_prob
                }
            };
            if accept {
                // accept candidate
                current_score = new_score;
            } else {
                // reject candidate
                solution.0[i_chosen] = old;
            }

            if current_score < best_score {
                best_score = current_score;
                best_solution = solution.clone();
            }
        }
    }
}
impl AnnealingAI {
    fn calc_ann_score(&self, program: &Program, image: &Image, state: &State) -> Result<f64, ProgramExecError> {
        Ok(calc_score(program, image, state)? as f64)
    }
}
