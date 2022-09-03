use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::calc_score;
use crate::simulator::simulate;
use crate::simulator::simulate_partial;
use crate::simulator::SimpleBlock;
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
        for iter in 0..100 {
            let mut next_program = prev_program.clone();
            let t = rng.gen_range(0..next_program.0.len());
            let mut state =
                simulator::State::initial_state(image.width() as i32, image.height() as i32);
            simulate_partial(&mut state, &prev_program.0[0..t]).unwrap();
            match next_program.0[t] {
                Move::PCut {
                    ref block_id,
                    point,
                } => {
                    let dx = rng.gen_range(-2..=2);
                    let dy = rng.gen_range(-2..=2);
                    let npoint = Point::new(point.x + dx, point.y + dy);
                    next_program.0[t] = Move::PCut {
                        block_id: block_id.clone(),
                        point: npoint,
                    };
                    // TODO child adjust
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
            if let Some(score) = calc_score(&next_program, image) {
                if score < best_score {
                    best_score = score;
                    best_program = next_program.clone();
                    prev_program = next_program;
                }
            }
        }
        return best_program;
    }
}
