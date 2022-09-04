use std::collections::HashMap;

use crate::image::*;
use crate::isl::*;

#[derive(Debug, thiserror::Error)]
#[error("line {line_number}: {mv} is invalid")]
pub struct ProgramExecError {
    line_number: usize,
    mv: Move,
    block: Option<SimpleBlock>,
}

pub fn program_exec_error(line_number: usize, mv: Move, state: &State) -> ProgramExecError {
    let block = match mv {
        Move::LCut { ref block_id, .. } => state.blocks.get(block_id).cloned(),
        Move::PCut { ref block_id, .. } => state.blocks.get(block_id).cloned(),
        Move::Color { ref block_id, .. } => state.blocks.get(block_id).cloned(),
        _ => None,
    };
    ProgramExecError {
        line_number,
        mv,
        block,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockState {
    Active,
    Deleted,
    Merged,
}
impl BlockState {
    pub fn is_active(self) -> bool {
        self == BlockState::Active
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleBlock {
    pub p: Point,
    pub size: Point,
    pub color: Color,
    pub state: BlockState,
}
impl SimpleBlock {
    pub fn new(p: Point, size: glam::IVec2, color: Color) -> Self {
        SimpleBlock {
            p,
            size,
            color,
            state: BlockState::Active,
        }
    }
    #[allow(dead_code)]
    pub fn rasterize(&self, image: &mut Image) {
        let w = image.width();
        let h = image.height();
        self.partial_rasterize(glam::IVec2::ZERO, Point::new(w as i32, h as i32), image);
    }
    pub fn partial_rasterize(&self, p: Point, size: Point, image: &mut Image) {
        if self.state == BlockState::Deleted {
            return;
        }
        if self.color == INVALID_COLOR {
            return;
        }
        let w = std::cmp::min((p.x + size.x) as usize, image.width());
        let h = std::cmp::min((p.y + size.y) as usize, image.height());
        let t = std::cmp::max(p.y as usize, self.p.y as usize);
        let b = std::cmp::min(h, (self.p.y + self.size.y) as usize);
        let l = std::cmp::max(p.x as usize, self.p.x as usize);
        let r = std::cmp::min(w, (self.p.x + self.size.x) as usize);
        for y in t..b {
            for x in l..r {
                image.0[y][x] = self.color;
            }
        }
    }
    pub fn area(&self) -> i32 {
        self.size.x * self.size.y
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub blocks: HashMap<BlockId, SimpleBlock>,
    pub next_global_id: u32,
    pub cost_coeff_version: u8, // 0 or 1
}
impl State {
    pub fn initial_state(w: i32, h: i32, cost_coeff_version: u8) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId::new(&vec![0]),
            SimpleBlock::new(Point::new(0, 0), glam::IVec2::new(w, h), Color::ONE),
        );
        State {
            blocks,
            next_global_id: 1,
            cost_coeff_version,
        }
    }
    // 指定したブロックが1つだけ入ったStateを返す
    pub fn block_state(&self, block_id: BlockId, cost_coeff_version: u8) -> Self {
        let mut blocks = HashMap::new();
        let block = self.blocks[&block_id].clone();
        blocks.insert(block_id, block);
        State {
            blocks,
            next_global_id: self.next_global_id,
            cost_coeff_version,
        }
    }
    #[allow(dead_code)]
    pub fn sample_active_block(&self, rng: &mut impl rand::Rng) -> BlockId {
        let blocks = self
            .blocks
            .iter()
            .filter(|(_id, block)| block.state.is_active())
            .collect::<Vec<_>>();
        let t = rng.gen_range(0..blocks.len());
        return blocks[t].0.clone();
    }
}

pub fn merge_block(block1: &SimpleBlock, block2: &SimpleBlock) -> Option<SimpleBlock> {
    let mut block1 = block1.clone();
    let mut block2 = block2.clone();
    if block1.p.x > block2.p.x || block1.p.y > block2.p.y {
        std::mem::swap(&mut block1, &mut block2);
        // std::mem::swap(&mut a, &mut b);
    }
    let next_size = if block1.p.x == block2.p.x {
        if block1.size.x != block2.size.x || block1.p.y + block1.size.y != block2.p.y {
            return None;
        }
        Point::new(block1.size.x, block1.size.y + block2.size.y)
    } else {
        if block1.size.y != block2.size.y || block1.p.x + block1.size.x != block2.p.x {
            return None;
        }
        Point::new(block1.size.x + block2.size.x, block1.size.y)
    };
    // TODO Colorが混ざるので ComplexBlock にする
    let next_block = SimpleBlock::new(block1.p, next_size, INVALID_COLOR);
    return Some(next_block);
}

#[allow(dead_code)]
#[must_use]
pub fn simulate(state: &mut State, mv: &Move) -> Option<()> {
    match mv {
        Move::PCut {
            ref block_id,
            point,
        } => {
            let mut simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.p;
            let offset = *point - p;
            if offset.x <= 0
                || offset.x >= simple_block.size.x
                || offset.y <= 0
                || offset.y >= simple_block.size.y
            {
                return None;
            }
            let dx = [0, offset.x, offset.x, 0];
            let dy = [0, 0, offset.y, offset.y];
            let nw = [
                offset.x,
                simple_block.size.x - offset.x,
                simple_block.size.x - offset.x,
                offset.x,
            ];
            let nh = [
                offset.y,
                offset.y,
                simple_block.size.y - offset.y,
                simple_block.size.y - offset.y,
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
            simple_block.state = BlockState::Deleted;
            state.blocks.insert(block_id.clone(), simple_block);
        }
        Move::LCut {
            ref block_id,
            orientation,
            line_number,
        } => {
            let mut simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.p;
            let offset = match orientation {
                Orientation::Horizontal => *line_number - p.y,
                Orientation::Vertical => *line_number - p.x,
            };
            let mut dx = [0, 0];
            let mut dy = [0, 0];
            let mut nw = [simple_block.size.x, simple_block.size.x];
            let mut nh = [simple_block.size.y, simple_block.size.y];
            match orientation {
                Orientation::Horizontal => {
                    if offset <= 0 || simple_block.size.y <= offset {
                        return None;
                    }
                    dy = [0, offset];
                    nh = [offset, simple_block.size.y - offset];
                }
                Orientation::Vertical => {
                    if offset <= 0 || simple_block.size.x <= offset {
                        return None;
                    }
                    dx = [0, offset];
                    nw = [offset, simple_block.size.x - offset];
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
            simple_block.state = BlockState::Deleted;
            state.blocks.insert(block_id.clone(), simple_block);
        }
        Move::Color {
            ref block_id,
            color,
        } => {
            let mut simple_block = state.blocks.get_mut(block_id)?;
            simple_block.color = *color;
        }
        Move::Swap { ref a, ref b } => {
            let mut block1 = state.blocks.get(a)?.clone();
            let mut block2 = state.blocks.get(b)?.clone();
            if block1.size != block2.size {
                return None;
            }
            std::mem::swap(&mut block1.p, &mut block2.p);
            state.blocks.insert(a.clone(), block1);
            state.blocks.insert(b.clone(), block2);
        }
        Move::Merge { ref a, ref b } => {
            let mut block1 = state.blocks.get(a)?.clone();
            let mut block2 = state.blocks.get(b)?.clone();
            let next_block = merge_block(&block1, &block2)?;
            // TODO Colorが混ざるので ComplexBlock にする
            state
                .blocks
                .insert(BlockId::new(&vec![state.next_global_id]), next_block);
            state.next_global_id += 1;
            block1.state = BlockState::Merged;
            block2.state = BlockState::Merged;
            state.blocks.insert(a.clone(), block1);
            state.blocks.insert(b.clone(), block2);
        }
    }
    Some(())
}

pub fn simulate_all(program: &Program, initial_state: &State) -> Result<State, ProgramExecError> {
    let mut state = initial_state.clone();
    simulate_partial(&mut state, &program.0)?;
    Ok(state)
}

pub fn simulate_partial(state: &mut State, program: &[Move]) -> Result<(), ProgramExecError> {
    let mut line_number = 1;
    for mv in program {
        simulate(state, mv).ok_or_else(|| program_exec_error(line_number, mv.clone(), state))?;
        line_number += 1;
    }
    Ok(())
}

static COST_COEFF_TABLE: [[f32; 5]; 2] = [
    // PCut LCut Color Swap Merge
    [10.0, 7.0, 5.0, 3.0, 1.0],
    [3.0, 2.0, 5.0, 3.0, 1.0],
];

pub fn move_cost(state: &State, mv: &Move, w: usize, h: usize) -> Option<i64> {
    let (i, area) = match mv {
        Move::PCut { ref block_id, .. } => (0, state.blocks.get(block_id)?.area()),
        Move::LCut { ref block_id, .. } => (1, state.blocks.get(block_id)?.area()),
        Move::Color { ref block_id, .. } => (2, state.blocks.get(block_id)?.area()),
        Move::Swap { ref a, .. } => (3, state.blocks.get(a)?.area()),
        Move::Merge { ref a, ref b } => (
            4,
            state.blocks.get(a)?.area().max(state.blocks.get(b)?.area()),
        ),
    };
    let base = COST_COEFF_TABLE[state.cost_coeff_version as usize][i];
    Some((base * (w * h) as f32 / area as f32).round() as i64)
}

pub fn move_cost_without_state(
    mv: &Move,
    target_area: usize,
    w: usize,
    h: usize,
    cost_coeff_version: u8,
) -> i64 {
    assert!(target_area > 0);
    assert!(w > 0);
    assert!(h > 0);
    let i = match mv {
        Move::PCut { .. } => 0,
        Move::LCut { .. } => 1,
        Move::Color { .. } => 2,
        Move::Swap { .. } => 3,
        Move::Merge { .. } => 4,
    };
    let base = COST_COEFF_TABLE[cost_coeff_version as usize][i];
    (base * (w * h) as f32 / target_area as f32).round() as i64
}

#[allow(dead_code)]
pub fn rasterize_state(state: &State, w: usize, h: usize) -> Image {
    return rasterize_parital_state(
        Point::new(0, 0),
        Point::new(w as i32, h as i32),
        state,
        w,
        h,
    );
}

fn rasterize_parital_state(p: Point, size: Point, state: &State, w: usize, h: usize) -> Image {
    let mut image = Image::new(w, h);

    // ブロック a とブロック b がマージされてブロック c ができたとする。
    // c を描画するよりも前に a と b が描画されることを保証するために
    // ここでソートする
    let mut blocks = state.blocks.iter().collect::<Vec<_>>();
    blocks.sort_by(|(id1, _), (id2, _)| id1.cmp(id2));

    for (_, simple_block) in blocks {
        simple_block.partial_rasterize(p, size, &mut image);
    }
    image
}

pub fn calc_state_similarity(state: &State, target_image: &Image) -> i64 {
    let w = target_image.width();
    let h = target_image.height();
    return calc_partial_state_similarity(
        Point::new(0, 0),
        Point::new(w as i32, h as i32),
        state,
        target_image,
    );
}

pub fn calc_partial_state_similarity(
    p: Point,
    size: Point,
    state: &State,
    target_image: &Image,
) -> i64 {
    let w = target_image.width();
    let h = target_image.height();
    let current_image = rasterize_parital_state(p, size, state, w, h);
    let mut similarity: f64 = 0.0;
    for y in p.y..(p.y + size.y) {
        for x in p.x..(p.x + size.x) {
            let d =
                current_image.0[y as usize][x as usize] - target_image.0[y as usize][x as usize];
            similarity += (d * 255.0).round().length() as f64;
        }
    }
    return (similarity * 0.005).round() as i64;
}

// 単色で塗りつぶされている場合のsimilarityを計算する
pub fn calc_partial_one_color_similarity(
    p: Point,
    size: Point,
    color: Color,
    target_image: &Image,
) -> i64 {
    let mut similarity: f64 = 0.0;
    for y in p.y..std::cmp::min(p.y + size.y, target_image.height() as i32) {
        for x in p.x..std::cmp::min(p.x + size.x, target_image.width() as i32) {
            let d = color - target_image.0[y as usize][x as usize];
            similarity += (d * 255.0).round().length() as f64;
        }
    }
    return (similarity * 0.005).round() as i64;
}

#[allow(dead_code)]
pub fn calc_score(
    program: &Program,
    target_image: &Image,
    initial_state: &State,
) -> Result<i64, ProgramExecError> {
    let h = target_image.height();
    let w = target_image.width();
    let mut state = initial_state.clone();
    let mut cost = 0;
    for line_number in 0..program.0.len() {
        let mv = &program.0[line_number];
        cost += move_cost(&state, &mv, w, h)
            .ok_or_else(|| program_exec_error(line_number + 1, mv.clone(), &state))?;
        simulate(&mut state, mv)
            .ok_or_else(|| program_exec_error(line_number + 1, mv.clone(), &state))?;
    }
    cost += calc_state_similarity(&state, target_image);
    Ok(cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec2;

    #[test]
    fn test_simulate_pcut() {
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 0]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 2]),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 3]),
                color: Color::new(0.0, 0.0, 1.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "rr...",
            "bbggg",
            "bbggg",
        ]);

        let actual = rasterize_state(&state, 5, 3);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_pcut_twice() {
        let mut state = State::initial_state(8, 8, 0);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0]),
                point: Point::new(4, 4),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0, 1]),
                point: Point::new(6, 2),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 1, 0]),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "....gg..",
            "....gg..",
            "........",
            "........",
            "........",
            "........",
            "........",
            "........",
        ]);

        let actual = rasterize_state(&state, 8, 8);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_lcut_twice() {
        let mut state = State::initial_state(8, 8, 0);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![0]),
                orientation: Orientation::Horizontal,
                line_number: 4,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![0, 1]),
                orientation: Orientation::Horizontal,
                line_number: 6,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 1, 0]),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "........",
            "........",
            "........",
            "........",
            "gggggggg",
            "gggggggg",
            "........",
            "........",
        ]);

        let actual = rasterize_state(&state, 8, 8);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_swap() {
        let mut state = State::initial_state(4, 3, 0);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 0]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Swap {
                a: BlockId::new(&vec![0, 0]),
                b: BlockId::new(&vec![0, 1]),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "..rr",
            "..rr",
            "..rr",
        ]);

        let actual = rasterize_state(&state, 4, 3);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_merge_vertically() {
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Merge {
                a: BlockId::new(&vec![0, 0]),
                b: BlockId::new(&vec![0, 3]),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![1]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "rr...",
            "rr...",
            "rr...",
        ]);

        let actual = rasterize_state(&state, 5, 3);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_merge_horizontally() {
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Merge {
                a: BlockId::new(&vec![0, 2]),
                b: BlockId::new(&vec![0, 3]),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![1]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            ".....",
            "rrrrr",
            "rrrrr",
        ]);

        let actual = rasterize_state(&state, 5, 3);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simulate_merge_complex() {
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId::new(&vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 2]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 3]),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Merge {
                a: BlockId::new(&vec![0, 2]),
                b: BlockId::new(&vec![0, 3]),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![1]),
                orientation: Orientation::Horizontal,
                line_number: 2,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![1, 1]),
                color: Color::new(0.0, 0.0, 1.0, 1.0),
            },
        )
        .unwrap();

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            ".....",
            "ggrrr",
            "bbbbb",
        ]);

        let actual = rasterize_state(&state, 5, 3);

        eprint!("actual:\n{}", actual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simple_block_rasterize() {
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let simple_block = SimpleBlock::new(IVec2::new(1, 2), IVec2::new(5, 3), red);

        let mut image = Image::new(10, 4);
        simple_block.rasterize(&mut image);

        #[rustfmt::skip]
        let expected = Image::from_string_array(&[
            "..........",
            "..........",
            ".rrrrr....",
            ".rrrrr....",
        ]);

        //eprintln!("actuall_image:\n{}", image);

        assert_eq!(expected, image);
    }

    #[test]
    fn test_calc_state_similarity() {
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId::new(&vec![0, 1]),
                color: red,
            },
        )
        .unwrap();
        // ..rrr
        // ..rrr
        // ..rrr

        #[rustfmt::skip]
        let target_image = Image::from_string_array(&[
            "..rr.",
            "..r.r",
            ".rrrr",
        ]);

        //eprintln!("target_image:\n{}", target_image);
        //eprintln!("actuall_image:\n{}", rasterize_state(&state, 5, 3));
        //eprintln!("actuall_image:\n{:?}", rasterize_state(&state, 5, 3));

        let expected_pixel_diff = 3.0 * (255.0 * 255.0f32 + 255.0 * 255.0f32).sqrt();
        let expected_similarity = (expected_pixel_diff * 0.005).round() as i64;

        let actuall = calc_state_similarity(&state, &target_image);
        assert_eq!(expected_similarity, actuall);
    }

    #[test]
    fn test_move_cost() {
        //pub fn move_cost(state: &State, mv: &Move, w: usize, h: usize) -> Option<f32>
        let mut state = State::initial_state(5, 3, 0);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId::new(&vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        let mv = Move::Color {
            block_id: BlockId::new(&vec![0, 1]),
            color: Color::ZERO,
        };
        let actual = move_cost(&state, &mv, 5, 3).unwrap();
        let expected = (5.0f32 * (5.0 * 3.0) / (3.0 * 3.0)).round() as i64;
        assert_eq!(expected, actual);
    }

    #[test]
    fn reproduce_problem_8() {
        let image = crate::image::open("../problems/8.png").unwrap();
        let state = State::initial_state(400, 400, 0);
        let program = vec![
            Move::PCut { block_id: BlockId::new(&vec![0]), point: Point::new(37, 233) },
            Move::LCut { block_id: BlockId::new(&vec![0, 2]), orientation: Orientation::Vertical, line_number: 354 },
            Move::Color { block_id: BlockId::new(&vec![0, 2, 0]), color: Color::ZERO },
            Move::Color { block_id: BlockId::new(&vec![0, 1]), color: Color::new(254.0, 254.0, 254.0, 255.0) / 255.0 },
        ];
        let score = calc_score(&Program(program), &image, &state).unwrap();
        assert_eq!(138571, score);
    }
}
