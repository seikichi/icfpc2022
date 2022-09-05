use crate::ai;
use crate::image;
use crate::image::Image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::calc_partial_image_similarity;
use crate::simulator::rasterize_parital_state;
use crate::simulator::rasterize_state;
use crate::simulator::simulate_all;
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
    pub initial_temperature: f64,
    pub dp_divide_max: usize,
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
        let (mut current_end_state, mut current_move_score) =
            simulate_all(&prev_program, initial_state, image.width(), image.height()).unwrap();
        let mut current_image = rasterize_state(&current_end_state, image.width(), image.height());

        let mut best_program = initial_program.clone();
        let mut best_score = current_score;

        let mut temperature;

        for iter in 0..self.n_iters {
            // tweak temperature
            let progress = (iter as f64) / (self.n_iters as f64);
            temperature = self.initial_temperature * (1.0 - progress) * (-progress).exp2();

            if prev_program.0.len() == 0 {
                break;
            }
            let (candidate_program, lt, size, description) = match self.make_neighbor(
                &prev_program,
                image,
                initial_state,
                &current_end_state,
                &mut rng,
            ) {
                Some(x) => x,
                None => continue,
            };
            let (candidate_end_state, candidate_move_score) = match simulate_all(
                &candidate_program,
                initial_state,
                image.width(),
                image.height(),
            ) {
                Ok(result) => result,
                Err(_err) => {
                    // assert!(!description.contains("DpAI"));
                    // log::debug!("{} {}", description, _err);
                    continue;
                }
            };
            let candidate_partial_image = rasterize_parital_state(
                lt,
                size,
                &candidate_end_state,
                image.width(),
                image.height(),
            );
            let diff_image_score_from =
                calc_partial_image_similarity(lt, size, &current_image, image);
            let diff_image_score_to =
                calc_partial_image_similarity(lt, size, &candidate_partial_image, image);
            let new_score = candidate_move_score
                + diff_image_score_to
                + (current_score - diff_image_score_from - current_move_score);

            // let true_score = calc_score(&candidate_program, image, initial_state).unwrap();
            // println!("{} {} {}", description, new_score, true_score);
            // assert!((true_score - new_score).abs() < 2);

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
                        // log::debug!("{} {:.4} {}", delta, accept_prob, description);
                        rng.gen::<f64>() < accept_prob
                    }
                }
            };

            if accept {
                info!("iter: {:3}, score: {:7} {}", iter, new_score, description);
                prev_program = candidate_program;
                (current_end_state, current_move_score) =
                    simulate_all(&prev_program, initial_state, image.width(), image.height())
                        .unwrap();
                for y in lt.y..(lt.y + size.y) {
                    for x in lt.x..(lt.x + size.x) {
                        current_image.0[y as usize][x as usize] =
                            candidate_partial_image.0[y as usize][x as usize];
                    }
                }
                // round分の誤差が出るので計算しなおす
                current_score = current_move_score
                    + calc_partial_image_similarity(
                        Point::new(0, 0),
                        Point::new(image.width() as i32, image.height() as i32),
                        &current_image,
                        image,
                    );

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
        end_state: &State,
        rng: &mut impl Rng,
    ) -> Option<(Program, Point, Point, String)> {
        let description;
        let mut next_program = prev_program.clone();
        let tl: Point;
        let size: Point;
        if rng.gen_range(0..100) == 0 {
            // 1/100 の確率でランダムにDpAIで分割する
            let block_id = end_state.sample_active_block(rng);
            let tl = end_state.blocks[&block_id].p;
            let size = end_state.blocks[&block_id].size;
            let mut next_program = self.solve_by_dp_ai_one_block(
                next_program,
                &block_id,
                image,
                initial_state,
                end_state,
                rng,
            );
            if prev_program.len() == next_program.len() {
                return None;
            }
            next_program.remove_redundant_color_move();
            description = format!("Divide by DpAI: {}", block_id);
            return Some((next_program, tl, size, description));
        }
        let t = rng.gen_range(0..next_program.0.len());
        let mv = next_program.0[t].clone();
        match mv {
            Move::PCut {
                ref block_id,
                point,
            } => {
                let block = end_state.blocks[block_id];
                tl = block.p;
                size = block.size;
                let r = rng.gen_range(0..8);
                if r > 0 {
                    // change PCut point
                    let dx = rng.gen_range(-5..=5);
                    let dy = rng.gen_range(-5..=5);
                    if dx == 0 || dy == 0 {
                        return None;
                    }
                    let npoint = Point::new(point.x + dx, point.y + dy);
                    next_program.0[t] = Move::PCut {
                        block_id: block_id.clone(),
                        point: npoint,
                    };
                    description = format!("move PCut dx:{} dy:{} {}", dx, dy, next_program.0[t]);
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
                        end_state,
                        rng,
                    );
                    next_program.remove_redundant_color_move();
                    description = format!("Remove PCut & divide by DpAI");
                }
            }
            Move::LCut {
                ref block_id,
                orientation,
                line_number,
            } => {
                let block = end_state.blocks[block_id];
                tl = block.p;
                size = block.size;
                let r = rng.gen_range(0..8);
                if r > 0 {
                    // change LCut position
                    let d = rng.gen_range(-5..=5);
                    if d == 0 {
                        return None;
                    }
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
                        end_state,
                        rng,
                    );
                    next_program.remove_redundant_color_move();
                    description = format!("Remove LCut & divide by DpAI");
                }
            }
            Move::Color {
                ref block_id,
                color: prev_color,
            } => {
                let block = end_state.blocks[block_id];
                tl = block.p;
                size = block.size;
                let r = rng.gen_range(0..5);
                if r > 0 {
                    let color = if r <= 2 {
                        // random sampling
                        let x = block.p.x + rng.gen_range(0..block.size.x);
                        let y = block.p.y + rng.gen_range(0..block.size.y);
                        image.0[y as usize][x as usize]
                    } else {
                        // average
                        image.average(block.p, block.size)
                    };
                    let d = prev_color - color;
                    let similarity = (d * 255.0).round().length() as f64;
                    if similarity < 1.5 {
                        return None;
                    }
                    next_program.0[t] = Move::Color {
                        block_id: block_id.clone(),
                        color,
                    };
                    description = format!(
                        "change Color: {} -> {}",
                        prev_color * 255.0,
                        next_program.0[t]
                    );
                } else {
                    description = format!("remove Color: {}", next_program.0[t]);
                    next_program.0.remove(t);
                }
            }
            _ => {
                // Do nothing
                return None;
            }
        }
        Some((next_program, tl, size, description))
    }

    fn solve_by_dp_ai_one_block(
        &self,
        program: Program,
        block_id: &BlockId,
        image: &Image,
        initial_state: &State,
        end_state: &State,
        rng: &mut impl Rng,
    ) -> Program {
        let mut program = program;
        let d = rng.gen_range(4..=self.dp_divide_max);
        let c = rng.gen_range(3..=8);
        let temp_state = end_state.block_state(block_id.clone(), initial_state.cost_coeff_version);
        let mut dp_ai = ai::DpAI::new(d, c);
        let mut dp_program = dp_ai.solve(image, &temp_state);
        program.0.append(&mut dp_program.0);
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
