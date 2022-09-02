use std::fmt::Display;

use glam::{IVec2, Vec4};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub Vec<u32>);
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

pub type Point = IVec2;
fn format_point(p: &Point) -> String {
    format!("[{}, {}]", p.x, p.y)
}

pub type Color = Vec4;
fn format_color(c: &Color) -> String {
    format!(
        "[{}, {}, {}, {}]",
        (c.x * 255.0).round() as u32,
        (c.y * 255.0).round() as u32,
        (c.z * 255.0).round() as u32,
        (c.w * 255.0).round() as u32
    )
}

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

#[test]
fn move_display_test() {
    // TODO
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Move>);

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for m in &self.0 {
            writeln!(f, "{m}");
        }
        Ok(())
    }
}
