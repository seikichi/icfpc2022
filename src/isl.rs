
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block(pub Vec<u32>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point { pub x: u32, pub y: u32}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation { Vertical, Horizontal }

#[derive(Debug, Clone, PartialEq)]
pub enum Move {
    PCut { block: Block, point: Point },
    LCut { block: Block, orientation: Orientation, linue_number: i32},
    Color { block:Block, color: Color },
    Swap { a: Block, b: Block },
    Merge { a: Block, b: Block },
}

// いる？
pub struct Program(Vec<Move>);

