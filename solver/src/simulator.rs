use std::collections::HashMap;

use crate::image::*;
use crate::isl::*;

#[derive(Debug, thiserror::Error)]
#[error("line {line_number}: {mv} is invalid")]
pub struct ProgramExecError {
    line_number: usize,
    mv: Move,
}

pub fn program_exec_error(line_number: usize, mv: Move) -> ProgramExecError {
    ProgramExecError { line_number, mv }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleBlock {
    pub p: Point,
    pub size: Point,
    pub color: Color,
    pub active: bool,
}
impl SimpleBlock {
    pub fn new(p: Point, size: glam::IVec2, color: Color) -> Self {
        SimpleBlock {
            p,
            size,
            color,
            active: true,
        }
    }
    #[allow(dead_code)]
    pub fn rasterize(&self, image: &mut Image) {
        let w = image.width();
        let h = image.height();
        self.partial_rasterize(glam::IVec2::ZERO, Point::new(w as i32, h as i32), image);
    }
    pub fn partial_rasterize(&self, p: Point, size: Point, image: &mut Image) {
        if !self.active {
            return;
        }
        if self.color == INVALID_COLOR {
            panic!("This block has INVALID_COLOR, which cannot be rasterized");
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub blocks: HashMap<BlockId, SimpleBlock>,
    pub next_global_id: u32,
}
impl State {
    pub fn initial_state(w: i32, h: i32) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId(vec![0]),
            SimpleBlock::new(Point::new(0, 0), glam::IVec2::new(w, h), Color::ONE),
        );
        State {
            blocks,
            next_global_id: 1,
        }
    }
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
            simple_block.active = false;
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
            simple_block.active = false;
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
            let mut a = a;
            let mut b = b;
            let mut block1 = state.blocks.get(a)?.clone();
            let mut block2 = state.blocks.get(b)?.clone();
            if block1.p.x > block2.p.x || block1.p.y > block2.p.y {
                std::mem::swap(&mut block1, &mut block2);
                std::mem::swap(&mut a, &mut b);
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
            state
                .blocks
                .insert(BlockId(vec![state.next_global_id]), next_block);
            state.next_global_id += 1;
            block1.active = false;
            block2.active = false;
            state.blocks.insert(a.clone(), block1);
            state.blocks.insert(b.clone(), block2);
        }
    }
    Some(())
}

pub fn simulate_all(program: &Program, img: &Image) -> Result<State, ProgramExecError> {
    let mut state = State::initial_state(img.width() as i32, img.height() as i32);
    simulate_partial(&mut state, &program.0)?;
    Ok(state)
}

pub fn simulate_partial(state: &mut State, program: &[Move]) -> Result<(), ProgramExecError> {
    let mut line_number = 1;
    for mv in program {
        simulate(state, mv).ok_or_else(|| program_exec_error(line_number, mv.clone()))?;
        line_number += 1;
    }
    Ok(())
}

pub fn move_cost(state: &State, mv: &Move, w: usize, h: usize) -> Option<i64> {
    let (base, size) = match mv {
        Move::PCut {
            ref block_id,
            point: _,
        } => (10.0, state.blocks.get(block_id)?.size),
        Move::LCut {
            ref block_id,
            orientation: _,
            line_number: _,
        } => (7.0, state.blocks.get(block_id)?.size),
        Move::Color {
            ref block_id,
            color: _,
        } => (5.0, state.blocks.get(block_id)?.size),
        Move::Swap { ref a, b: _ } => (3.0, state.blocks.get(a)?.size),
        Move::Merge {
            a: ref _a,
            b: ref _b,
        } => {
            unimplemented!()
        }
    };
    Some((base * (w * h) as f32 / (size.x * size.y) as f32).round() as i64)
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
    for (_, simple_block) in &state.blocks {
        simple_block.partial_rasterize(p, size, &mut image);
    }
    return image;
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

#[allow(dead_code)]
pub fn calc_score(program: &Program, target_image: &Image) -> Result<i64, ProgramExecError> {
    let h = target_image.height();
    let w = target_image.width();
    let mut state = State::initial_state(w as i32, h as i32);
    let mut cost = 0;
    for line_number in 0..program.0.len() {
        let mv = &program.0[line_number];
        cost += move_cost(&state, &mv, w, h)
            .ok_or_else(|| program_exec_error(line_number + 1, mv.clone()))?;
        simulate(&mut state, mv).ok_or_else(|| program_exec_error(line_number + 1, mv.clone()))?;
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
        let mut state = State::initial_state(5, 3);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId(vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 0]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 2]),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 3]),
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
        let mut state = State::initial_state(8, 8);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId(vec![0]),
                point: Point::new(4, 4),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId(vec![0, 1]),
                point: Point::new(6, 2),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 1, 0]),
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
        let mut state = State::initial_state(8, 8);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId(vec![0]),
                orientation: Orientation::Horizontal,
                line_number: 4,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId(vec![0, 1]),
                orientation: Orientation::Horizontal,
                line_number: 6,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 1, 0]),
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
        let mut state = State::initial_state(4, 3);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId(vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 0]),
                color: Color::new(1.0, 0.0, 0.0, 1.0),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Swap {
                a: BlockId(vec![0, 0]),
                b: BlockId(vec![0, 1]),
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
        let mut state = State::initial_state(5, 3);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId(vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Merge {
                a: BlockId(vec![0, 0]),
                b: BlockId(vec![0, 3]),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![1]),
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
        let mut state = State::initial_state(5, 3);
        simulate(
            &mut state,
            &Move::PCut {
                block_id: BlockId(vec![0]),
                point: Point::new(2, 1),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Merge {
                a: BlockId(vec![0, 2]),
                b: BlockId(vec![0, 3]),
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![1]),
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
        let mut state = State::initial_state(5, 3);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId(vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        simulate(
            &mut state,
            &Move::Color {
                block_id: BlockId(vec![0, 1]),
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
        let mut state = State::initial_state(5, 3);
        simulate(
            &mut state,
            &Move::LCut {
                block_id: BlockId(vec![0]),
                orientation: Orientation::Vertical,
                line_number: 2,
            },
        )
        .unwrap();
        let mv = Move::Color {
            block_id: BlockId(vec![0, 1]),
            color: Color::ZERO,
        };
        let actual = move_cost(&state, &mv, 5, 3).unwrap();
        let expected = (5.0f32 * (5.0 * 3.0) / (3.0 * 3.0)).round() as i64;
        assert_eq!(expected, actual);
    }
}
