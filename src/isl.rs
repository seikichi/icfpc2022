use std::fmt::Display;

use glam::{IVec2, Vec4};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block(pub Vec<u32>);
impl Display for Block {
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
            Self::Vertical => write!(f, "[Y]"),
            Self::Horizontal => write!(f, "[X]"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Move {
    PCut {
        block: Block,
        point: Point,
    },
    LCut {
        block: Block,
        orientation: Orientation,
        line_number: i32,
    },
    Color {
        block: Block,
        color: Color,
    },
    Swap {
        a: Block,
        b: Block,
    },
    Merge {
        a: Block,
        b: Block,
    },
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::PCut { ref block, point } => {
                write!(f, "cut {} {}", block, format_point(point))
            }
            Move::LCut {
                ref block,
                orientation,
                line_number,
            } => {
                write!(f, "cut {block} {orientation} {line_number}")
            }
            Move::Color { ref block, color } => {
                write!(f, "color {} {}", block, format_color(color))
            }
            Move::Swap { ref a, ref b } => {
                write!(f, "color {a} {b}")
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
            write!(f, "{m}");
        }
        Ok(())
    }
}

