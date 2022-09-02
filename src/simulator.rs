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
    pub fn rasterize(&self, image: &mut Image) {
        let w = image.0[0].len();
        let h = image.0.len();
        let t = std::cmp::max(0, self.p.y as usize);
        let b = std::cmp::min(h, (self.p.y + self.size.y) as usize);
        let l = std::cmp::max(0, self.p.x as usize);
        let r = std::cmp::min(w, (self.p.x + self.size.x) as usize);
        for y in t..b {
            for x in l..r {
                image.0[y][x] = self.color;
            }
        }
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

fn rasterize_state(state: &State, w: usize, h: usize) -> Image {
    let mut image = Image(vec![vec![glam::Vec4::ZERO; w]; h]);
    for (_, simple_block) in &state.blocks {
        simple_block.rasterize(&mut image);
    }
    return image;
}

fn calc_state_similarity(state: &State, target_image: &Image) {
    let mut current_image = target_image.clone();

    // TODO
}

fn calc_score(program: &Program) {
    // TODO
}

#[test]
fn test_simulate() {
    // TODO
}
