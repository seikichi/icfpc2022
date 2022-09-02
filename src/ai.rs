use crate ::image;
use crate ::isl;

pub struct OneColorAI {
}

pub struct GridAI {
    pub rows: usize,
    pub cols: usize,
}

impl OneColorAI {
    pub fn solve(&self, image: &image::Image) -> isl::Program {
        let mut r_sum = 0.0f32;
        let mut g_sum = 0.0f32;
        let mut b_sum = 0.0f32;
        let mut a_sum = 0.0f32;

        for row in &image.0 {
            for color in row {
                r_sum += color.r;
                g_sum += color.g;
                b_sum += color.b;
                a_sum += color.a;
            }
        }

        let area = (image.0.len() * image.0[0].len()) as f32;
        let color = isl::Color {
            r: r_sum / area,
            g: g_sum / area,
            b: b_sum / area,
            a: a_sum / area,
        };

        isl::Program(vec![isl::Move::Color { block: isl::Block(vec![0]), color }])
    }
}

impl GridAI {
    pub fn solve(&self, image: &image::Image) -> isl::Program {
        let height = image.0.len();
        let width = image.0[0].len();

        // y 軸で切ってく...


        isl::Program(vec![])
    }
}



