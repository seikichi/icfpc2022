use crate::ai::HeadAI;
use crate::image;
use crate::isl;
use crate::simulator;

pub struct OneColorAI {}

impl HeadAI for OneColorAI {
    fn solve(&mut self, image: &image::Image, _initial_state: &simulator::State) -> isl::Program {
        let mut sum = glam::Vec4::ZERO;

        for row in &image.0 {
            for color in row {
                sum += *color;
            }
        }

        let color = sum / image.area() as f32;

        isl::Program(vec![isl::Move::Color {
            block_id: isl::BlockId::new(&vec![0]),
            color,
        }])
    }
}
