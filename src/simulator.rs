use std::collections::HashMap;

use crate::image::*;
use crate::isl::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub p: Point,
    pub size: Point,
}
impl Shape {
    pub fn new(p: Point, size: Point) -> Self {
        Shape { p, size }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleBlock {
    pub shape: Shape,
    pub color: Color,
}
impl SimpleBlock {
    pub fn new(shape: Shape, color: Color) -> Self {
        SimpleBlock { shape, color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub blocks: HashMap<BlockId, SimpleBlock>,
}

pub fn simulate(state: &mut State, mv: &Move) -> Option<()> {
    match mv {
        Move::PCut { ref block_id, point } => {
            let simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.shape.p;
            if point.x >= simple_block.shape.size.x || point.y >= simple_block.shape.size.y {
                return None;
            }
            let dx = [0, point.x, point.x, 0];
            let dy = [0, 0, point.y, point.y];
            let nw = [
                point.x,
                simple_block.shape.size.x - point.x,
                simple_block.shape.size.x - point.x,
                point.x,
            ];
            let nh = [
                point.y,
                point.y,
                simple_block.shape.size.y - point.y,
                simple_block.shape.size.y - point.y,
            ];
            for i in 0..4 {
                let nx = p.x + dx[i];
                let ny = p.y + dy[i];
                let mut next_id = block_id.clone();
                next_id.0.push(i as u32);
                let next_simple_block = SimpleBlock::new(
                    Shape::new(Point::new(nx, ny), Point::new(nw[i], nh[i])),
                    simple_block.color,
                );
                state.blocks.insert(next_id, next_simple_block);
            }
            state.blocks.remove(block_id);
        }
        Move::LCut {
            ref block_id,
            orientation,
            line_number,
        } => {
            let simple_block = state.blocks.get(block_id)?;
            match orientation {
                Orientation::Vertical => {}
                Orientation::Horizontal => {}
            }
            /*
            if point.x >= simple_block.shape.size.x || point.y >= simple_block.shape.size.y {
                return None;
            }
            */
            // TODO
            state.blocks.remove(block_id);
        }
        Move::Color { ref block_id, color } => {
            unimplemented!()
        }
        Move::Swap { ref a, ref b } => {
            unimplemented!()
        }
        Move::Merge { ref a, ref b } => {
            unimplemented!()
        }
    }
    Some(())
}
