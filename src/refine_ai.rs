use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::calc_score;
use crate::simulator::simulate;
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
