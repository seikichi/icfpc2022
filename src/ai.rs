use crate ::image;
use crate ::isl;

pub struct OneColorAI {
}

impl OneColorAI {
    pub fn solve(&self, image: &image::Image) -> isl::Program {
        let mut sum = glam::Vec4::ZERO;

        for row in &image.0 {
            for color in row {
                sum += *color;
            }
        }

        let area = (image.0.len() * image.0[0].len()) as f32;
        let color = sum / area;

        isl::Program(vec![isl::Move::Color { block: isl::Block(vec![0]), color }])
    }
}
