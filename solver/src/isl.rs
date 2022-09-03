use std::{collections::VecDeque, fmt::Display};

use glam::{IVec2, Vec4};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub VecDeque<u32>);
impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();
        for i in 0..self.0.len() {
            if i != 0 {
                ret.push('.');
            }
            ret.push_str(&self.0[i].to_string());
        }
        write!(f, "[{ret}]")
    }
}
impl BlockId {
    pub fn new(id: &[u32]) -> Self {
        BlockId(VecDeque::from_iter(id.iter().copied()))
    }
    pub fn is_child(&self, target: &BlockId) -> bool {
        if self.0.len() >= target.0.len() {
            return false;
        }
        for i in 0..self.0.len() {
            if self.0[i] != target.0[i] {
                return false;
            }
        }
        return true;
    }
}

pub type Point = IVec2;
pub fn format_point(p: &Point) -> String {
    format!("[{}, {}]", p.x, p.y)
}

pub type Color = Vec4;
pub fn format_color(c: &Color) -> String {
    format!(
        "[{}, {}, {}, {}]",
        (c.x * 255.0).round() as u32,
        (c.y * 255.0).round() as u32,
        (c.z * 255.0).round() as u32,
        (c.w * 255.0).round() as u32
    )
}
pub const INVALID_COLOR: Color = Color::new(-1.0, -1.0, -1.0, -1.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}
impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertical => write!(f, "[X]"),
            Self::Horizontal => write!(f, "[Y]"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Move {
    PCut {
        block_id: BlockId,
        point: Point,
    },
    LCut {
        block_id: BlockId,
        orientation: Orientation,
        line_number: i32,
    },
    Color {
        block_id: BlockId,
        color: Color,
    },
    Swap {
        a: BlockId,
        b: BlockId,
    },
    Merge {
        a: BlockId,
        b: BlockId,
    },
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::PCut {
                ref block_id,
                point,
            } => {
                write!(f, "cut {} {}", block_id, format_point(point))
            }
            Move::LCut {
                ref block_id,
                orientation,
                line_number,
            } => {
                write!(f, "cut {block_id} {orientation} [{line_number}]")
            }
            Move::Color {
                ref block_id,
                color,
            } => {
                write!(f, "color {} {}", block_id, format_color(color))
            }
            Move::Swap { ref a, ref b } => {
                write!(f, "swap {a} {b}")
            }
            Move::Merge { ref a, ref b } => {
                write!(f, "merge {a} {b}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_display_test() {
        let pcut = Move::PCut {
            block_id: BlockId::new(&vec![0, 4, 2]),
            point: IVec2::new(12, 34),
        };
        assert_eq!("cut [0.4.2] [12, 34]", format!("{}", pcut));

        let lcut = Move::LCut {
            block_id: BlockId::new(&vec![0, 4, 2]),
            orientation: Orientation::Horizontal,
            line_number: 3,
        };
        assert_eq!("cut [0.4.2] [Y] [3]", format!("{}", lcut));

        let color = Move::Color {
            block_id: BlockId::new(&vec![0, 4, 2]),
            color: Color::new(1.0, 1.0, 0.5, 1.0),
        };
        assert_eq!("color [0.4.2] [255, 255, 128, 255]", format!("{}", color));

        let swap = Move::Swap {
            a: BlockId::new(&vec![0, 4, 2]),
            b: BlockId::new(&vec![1]),
        };
        assert_eq!("swap [0.4.2] [1]", format!("{}", swap));

        let merge = Move::Merge {
            a: BlockId::new(&vec![0, 4, 2]),
            b: BlockId::new(&vec![1]),
        };
        assert_eq!("merge [0.4.2] [1]", format!("{}", merge));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Move>);
impl Display for Program {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for m in &self.0 {
            writeln!(f, "{m}")?;
        }
        Ok(())
    }
}
impl Program {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        return self.0.len();
    }
    #[allow(dead_code)]
    pub fn convert_initial_block_id(&mut self, id: u32) {
        for mv in self.0.iter_mut() {
            match mv {
                Move::PCut { block_id, .. } => {
                    block_id.0[0] = id;
                }
                Move::LCut { block_id, .. } => {
                    block_id.0[0] = id;
                }
                Move::Color { block_id, .. } => {
                    block_id.0[0] = id;
                }
                Move::Swap { a, b } => {
                    a.0[0] = id;
                    b.0[0] = id;
                }
                Move::Merge { a, b } => {
                    panic!("Can't use Merge")
                }
            }
        }
    }
}
