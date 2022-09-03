use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::calc_score;
use crate::simulator::simulate_partial;
use rand::Rng;

pub struct RefineAi {
    //
}

impl RefineAi {
    pub fn new() -> Self {
        RefineAi {}
    }
    pub fn solve(&self, initial_program: &Program, image: &image::Image) -> Program {
        // TODO seed_from_u64
        let mut rng = rand::thread_rng();
        let mut best_program = initial_program.clone();
        let mut best_score = simulator::calc_score(initial_program, image).unwrap();
        let mut prev_program = initial_program.clone();
        for iter in 0..50000 {
            let mut next_program = prev_program.clone();
            let t = rng.gen_range(0..next_program.0.len());
            let mv = next_program.0[t].clone();
            match mv {
                // TODO swap color timing
                Move::PCut {
                    ref block_id,
                    point,
                } => {
                    let r = rng.gen_range(0..2);
                    if r == 0 {
                        // change PCut point
                        let dx = rng.gen_range(-5..=5);
                        let dy = rng.gen_range(-5..=5);
                        let npoint = Point::new(point.x + dx, point.y + dy);
                        next_program.0[t] = Move::PCut {
                            block_id: block_id.clone(),
                            point: npoint,
                        };
                    } else if r == 1 {
                        next_program.0.remove(t);
                        if let Some(result) = Self::remove_all_child(&next_program, block_id) {
                            next_program = result;
                        } else {
                            continue;
                        }
                    } else {
                        // // PCut -> LCut and remove all child
                        // let (orientation, line_number) = if rng.gen_range(0..2) == 0 {
                        //     (Orientation::Vertical, point.x)
                        // } else {
                        //     (Orientation::Horizontal, point.y)
                        // };
                    }
                }
                Move::LCut {
                    ref block_id,
                    orientation,
                    line_number,
                } => {
                    // TODO
                    continue;
                }
                Move::Color {
                    ref block_id,
                    color,
                } => {
                    let mut state = simulator::State::initial_state(
                        image.width() as i32,
                        image.height() as i32,
                    );
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
                    next_program.0[t] = Move::Color {
                        block_id: block_id.clone(),
                        color,
                    };
                }
                _ => {
                    // Do nothing
                    continue;
                }
            }
            if let Ok(score) = calc_score(&next_program, image) {
                if score < best_score {
                    println!(
                        "iter: {:3}, score: {:7}, move: {}",
                        iter, score, prev_program.0[t]
                    );
                    best_score = score;
                    best_program = next_program.clone();
                    prev_program = next_program;
                }
            }
        }
        return best_program;
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
