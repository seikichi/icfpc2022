use std::collections::HashMap;

use crate::image::*;
use crate::isl::*;

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
}
impl State {
    pub fn initial_state(w: i32, h: i32) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId(vec![0]),
            SimpleBlock::new(Point::new(0, 0), glam::IVec2::new(w, h), Color::ONE),
        );
        State { blocks }
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
            simple_block.active = false;
            state.blocks.insert(block_id.clone(), simple_block);
        }
        Move::LCut {
            ref block_id,
            orientation,
            line_number,
        } => {
            let line_number = *line_number;
            let mut simple_block = state.blocks.get(block_id)?.clone();
            let p = simple_block.p;
            let mut dx = [0, 0];
            let mut dy = [0, 0];
            let mut nw = [simple_block.size.x, simple_block.size.x];
            let mut nh = [simple_block.size.y, simple_block.size.y];
            match orientation {
                Orientation::Horizontal => {
                    if line_number <= 0 || simple_block.size.y <= line_number {
                        return None;
                    }
                    dy = [0, line_number];
                    nh = [line_number, simple_block.size.y - line_number];
                }
                Orientation::Vertical => {
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
            let mut block1 = state.blocks.get_mut(a)?.clone();
            let mut block2 = state.blocks.get_mut(b)?.clone();
            if block1.size != block2.size {
                return None;
            }
            std::mem::swap(&mut block1.p, &mut block2.p);
            state.blocks.insert(a.clone(), block1);
            state.blocks.insert(b.clone(), block2);
        }
        Move::Merge {
            a: ref _a,
            b: ref _b,
        } => {
            unimplemented!()
        }
    }
    Some(())
}

pub fn simulate_all(program: &Program, img: &Image) -> Option<State> {
    let mut state = State::initial_state(img.width() as i32, img.height() as i32);
    for line_number in 0..program.0.len() {
        let mv = &program.0[line_number];
        let result = simulate(&mut state, mv);
        if result.is_none() {
            eprintln!("line {}: {} is invalid", line_number, mv);
            return None;
        }
    }
    return Some(state);
}

pub fn move_cost(state: &State, mv: &Move, w: usize, h: usize) -> Option<f32> {
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
    Some((base * (w * h) as f32 / (size.x * size.y) as f32).round())
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

pub fn calc_state_similarity(state: &State, target_image: &Image) -> f32 {
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
) -> f32 {
    let w = target_image.width();
    let h = target_image.height();
    let current_image = rasterize_parital_state(p, size, state, w, h);
    let mut similarity = 0.0;
    for y in p.y..(p.y + size.y) {
        for x in p.x..(p.x + size.x) {
            let d =
                current_image.0[y as usize][x as usize] - target_image.0[y as usize][x as usize];
            similarity += (d * 255.0).round().length();
        }
    }
    return similarity * 0.005;
}

#[allow(dead_code)]
pub fn calc_score(program: &Program, target_image: &Image) -> Option<f32> {
    let h = target_image.height();
    let w = target_image.width();
    let mut state = State::initial_state(w as i32, h as i32);
    let mut cost = 0.0;
    for line_number in 0..program.0.len() {
        let mv = &program.0[line_number];
        cost += move_cost(&state, &mv, w, h)?;
        let result = simulate(&mut state, mv);
        if result.is_none() {
            eprintln!("line {}: {} is invalid", line_number, mv);
            return None;
        }
    }
    cost += calc_state_similarity(&state, target_image);
    return Some(cost);
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
        let expected_similarity = expected_pixel_diff * 0.005;

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
        let expected = (5.0f32 * (5.0 * 3.0) / (3.0 * 3.0)).round();
        assert_eq!(expected, actual);
    }
}
