use crate::image;
use crate::isl;
use std::collection::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub p: Point,
    pub size: Point,
}
impl Shape {
    pub fn new(p: Point, size: Point) -> Self {
        Shape { p, size }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleBlock {
    pub shape: Shape,
    pub color: Color,
}
impl SimpleBlock {
    pub fn new(shape: Shape, color: Color) -> Self {
        SimpleBlock { shape, color }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub blocks: HashMap<Block, SimpleBlock>,
}
impl Simulator {
    pub fn simulate(&mut self, state: &State, mv: &Move) -> Option<()> {
        match mv {
            Move::PCut { ref block, point } => {
                let simple_block = self.blocks.get(block)?;
                if point.x >= simple_block.size.x || point.y >= simple_block.size.y {
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
                    let nx = simple_block.p.x + dx[i];
                    let ny = simple_block.p.y + dy[i];
                    let mut next_id = block.clone();
                    next_id.push(i);
                    let next_simple_block = SimpleBlock::new(
                        Shape::new(Point::new(nx, ny), Point::new(nw[i], nh[i])),
                        simple_block.color,
                    );
                    self.blocks.insert(next_id, next_simple_block);
                }
                self.blocks.remove(block);
            }
            Move::LCut {
                ref block,
                orientation,
                line_number,
            } => {
                let simple_block = self.blocks.get(block)?;
                match orientation {
                    Orientation::Vertical => {}
                    Orientation::Horizontal => {}
                }
                if point.x >= simple_block.size.x || point.y >= simple_block.size.y {
                    return None;
                }
                // TODO
                self.blocks.remove(block);
            }
            Move::Color { ref block, color } => {
                unimplmented!()
            }
            Move::Swap { ref a, ref b } => {
                unimplmented!()
            }
            Move::Merge { ref a, ref b } => {
                unimplmented!()
            }
        }
        return Some();
    }
}
