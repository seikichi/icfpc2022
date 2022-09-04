mod annealing;
mod changecolor;
mod cross;
mod dp;
mod grid;
mod merge;
mod onecolor;
mod rect;
mod refine;
mod swap;

pub use annealing::*;
pub use changecolor::*;
pub use cross::*;
pub use dp::*;
pub use grid::*;
pub use merge::*;
pub use onecolor::*;
pub use rect::*;
pub use refine::*;
pub use swap::*;

use crate::image;
use crate::isl;
use crate::simulator;

pub trait HeadAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> isl::Program;
}

pub trait ChainedAI {
    fn solve(
        &mut self,
        image: &image::Image,
        initial_state: &simulator::State,
        program: &isl::Program,
    ) -> isl::Program;
}
