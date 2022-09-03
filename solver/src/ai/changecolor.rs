use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct ChangeColorAI {}

impl HeadAI for ChangeColorAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> isl::Program {
        let mut programs = vec![];
        for (block_id, block) in initial_state.blocks.clone().iter() {
            let color = image.average(block.p, block.size);
            programs.push(isl::Move::Color {
                block_id: block_id.clone(),
                color,
            })
        }
        isl::Program(programs)
    }
}
