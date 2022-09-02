use std::fmt::Display;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}
impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Point { x, y }
    }
}
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            (self.r * 255.0).round() as u32,
            (self.g * 255.0).round() as u32,
            (self.b * 255.0).round() as u32,
            (self.a * 255.0).round() as u32
        )
    }
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
                write!(f, "cut {block} {point}")
            }
            Move::LCut {
                ref block,
                orientation,
                line_number,
            } => {
                write!(f, "cut {block} {orientation} {line_number}")
            }
            Move::Color { ref block, color } => {
                write!(f, "color {block} {color}")
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

// いる？
pub struct Program(Vec<Move>);

