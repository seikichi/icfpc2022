use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct ChangeColorAI {
    pub thresh: f32,
}

impl HeadAI for ChangeColorAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> isl::Program {
        let mut programs = vec![];
        for (block_id, block) in initial_state.blocks.clone().iter() {
            let color_ave = image.average(block.p, block.size);
            let color_occ = image.most_occurred(block.p, block.size);
            let next_move_ave = isl::Move::Color {
                block_id: block_id.clone(),
                color: color_ave,
            };
            let next_move_occ = isl::Move::Color {
                block_id: block_id.clone(),
                color: color_occ,
            };
            let sim_before = simulator::calc_partial_one_color_similarity(
                block.p,
                block.size,
                block.color,
                image,
            );
            let sim_ave =
                simulator::calc_partial_one_color_similarity(block.p, block.size, color_ave, image);
            let sim_occ =
                simulator::calc_partial_one_color_similarity(block.p, block.size, color_occ, image);
            let move_cost_ave =
                simulator::move_cost(initial_state, &next_move_ave, image.width(), image.height())
                    .unwrap();
            let move_cost_occ =
                simulator::move_cost(initial_state, &next_move_occ, image.width(), image.height())
                    .unwrap();
            if sim_ave + move_cost_ave >= sim_before && sim_occ + move_cost_occ >= sim_before {
                continue;
            } else if sim_ave + move_cost_ave >= sim_occ + move_cost_occ {
                programs.push(next_move_occ)
            } else {
                programs.push(next_move_ave)
            }
        }
        isl::Program(programs)
    }
}
