use std::collections::HashMap;

use crate::image::*;
use crate::isl::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleBlock {
    pub p: Point,
    pub size: Point,
    pub color: Color,
}
impl SimpleBlock {
    pub fn new(p: Point, size: glam::IVec2, color: Color) -> Self {
        SimpleBlock { p, size, color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub blocks: HashMap<BlockId, SimpleBlock>,
}

pub fn simulate(state: &mut State, mv: &Move) -> Option<()> {
    match mv {
        Move::PCut {
            ref block_id,
            point,
        } => {
            let simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.p;
            if point.x <= 0
                || point.x >= simple_block.size.x
                || point.y <= 0
                || point.y >= simple_block.size.y
            {
                return None;
            }
            let dx = [0, point.x, point.x, 0];
            let dy = [0, 0, point.y, point.y];
            let nw = [
                point.x,
                simple_block.size.x - point.x,
                simple_block.size.x - point.x,
                point.x,
            ];
            let nh = [
                point.y,
                point.y,
                simple_block.size.y - point.y,
                simple_block.size.y - point.y,
            ];
            for i in 0..4 {
                let nx = p.x + dx[i];
                let ny = p.y + dy[i];
                let mut next_id = block_id.clone();
                next_id.0.push(i as u32);
                let next_simple_block = SimpleBlock::new(
                    Point::new(nx, ny),
                    Point::new(nw[i], nh[i]),
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
            let line_number = *line_number;
            let simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.p;
            let mut dx = [0, 0];
            let mut dy = [0, 0];
            let mut nh = [simple_block.size.x, simple_block.size.x];
            let mut nw = [simple_block.size.y, simple_block.size.y];
            match orientation {
                Orientation::Vertical => {
                    if line_number <= 0 || simple_block.size.y <= line_number {
                        return None;
                    }
                    dy = [0, line_number];
                    nh = [line_number, simple_block.size.y - line_number];
                }
                Orientation::Horizontal => {
                    if line_number <= 0 || simple_block.size.x <= line_number {
                        return None;
                    }
                    dx = [0, line_number];
                    nw = [line_number, simple_block.size.x - line_number];
                }
            }
            for i in 0..2 {
                let nx = p.x + dx[i];
                let ny = p.y + dy[i];
                let mut next_id = block_id.clone();
                next_id.0.push(i as u32);
                let next_simple_block = SimpleBlock::new(
                    Point::new(nx, ny),
                    Point::new(nw[i], nh[i]),
                    simple_block.color,
                );
                state.blocks.insert(next_id, next_simple_block);
            }
            state.blocks.remove(block_id);
        }
        Move::Color {
            ref block_id,
            color,
        } => {
            let mut simple_block = state.blocks.get_mut(block_id)?;
            simple_block.color = *color;
        }
        Move::Swap { ref a, ref b } => {
            let block1 = state.blocks.get(a)?.clone();
            let block2 = state.blocks.get(b)?.clone();
            state.blocks.insert(a.clone(), block2);
            state.blocks.insert(b.clone(), block1);
        }
        Move::Merge { ref a, ref b } => {
            unimplemented!()
        }
    }
    Some(())
}
