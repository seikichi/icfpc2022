mod annealing;
mod cross;
mod dp;
mod grid;
mod onecolor;
mod refine;

pub use annealing::*;
pub use cross::*;
pub use dp::*;
pub use grid::*;
pub use onecolor::*;
pub use refine::*;

use crate::image;
use crate::isl;

pub trait HeadAI {
    fn solve(&mut self, image: &image::Image) -> isl::Program;
}

pub trait ChainedAI {
    fn solve(&mut self, image: &image::Image, program: &isl::Program) -> isl::Program;
}
