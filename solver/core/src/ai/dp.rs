use crate::ai::HeadAI;
use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::BlockState;
use crate::simulator::SimpleBlock;
use crate::simulator::State;
use arrayvec::ArrayVec;
use rand::rngs::ThreadRng;

use super::MergeAI;
#[derive(Debug, Clone, Copy)]
struct Child {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color_id: usize,
}
impl Child {
    fn new(x: usize, y: usize, w: usize, h: usize, color_id: usize) -> Self {
        Child {
            x,
            y,
            w,
            h,
            color_id,
        }
    }
}

pub struct DpAI {
    divide_num: usize,
    rng: ThreadRng,
    sample_color_num: usize,
    sampled_color: Vec<Color>,
    // memo[color_id][x][y][w][h] -> score
    // memo_restore[color_id][x][y][w][h] -> Some(今のブロックに対するProgram, 復元用の次の最適解))
    memo: Vec<Vec<Vec<Vec<Vec<i32>>>>>,
    memo_restore: Vec<Vec<Vec<Vec<Vec<Option<(ArrayVec<Move, 2>, ArrayVec<Child, 4>)>>>>>>,
    similality_memo: Vec<Vec<Vec<Option<i32>>>>,
    image: image::Image,
    initial_state: State,
    initial_block: SimpleBlock,
}

impl HeadAI for DpAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> Program {
        let d = self.divide_num;
        self.image = image.clone();
        let mut ret = Program(vec![]);
        let mut initial_block_id = initial_state.blocks.keys().next().unwrap().clone();
        self.initial_state = initial_state.clone();
        if initial_state.blocks.len() != 1 {
            let mut merge_ai = MergeAI::new(initial_state.cost_coeff_version);
            ret = merge_ai.solve(image, initial_state);
            initial_block_id = merge_ai.merged_block_id();
            self.initial_state = simulator::simulate_all(&ret, &self.initial_state).unwrap();
        }
        self.initial_block = self
            .initial_state
            .blocks
            .get(&initial_block_id)
            .unwrap()
            .clone();

        if self.width() < d || self.height() < d {
            return ret;
        }

        // color sampling
        self.sampled_color = image::k_means_color_sampling(
            image,
            self.sample_color_num - 1,
            20,
            self.topleft().x as usize,
            self.topleft().y as usize,
            self.width() as usize,
            self.height() as usize,
            &mut self.rng,
        );
        self.sampled_color.push(self.initial_block.color);
        self.sampled_color.reverse();
        // 画像の色数が sample_color_num より小さいような場合は
        // sampled_color が sample_color_num に満たない
        self.sample_color_num = self.sampled_color.len();

        // dp
        let _score = self.calc(0, 0, d, d, 0);
        let mut program = Program(vec![]);
        self.restore_program(&mut program, 0, 0, d, d, 0, &mut initial_block_id);

        ret.0.append(&mut program.0);
        return ret;
    }
}
impl DpAI {
    pub fn new(divide_num: usize, sample_color_num: usize) -> Self {
        let memo = vec![
            vec![
                vec![vec![vec![1 << 30; divide_num + 1]; divide_num + 1]; divide_num];
                divide_num
            ];
            sample_color_num
        ];
        let memo_restore =
            vec![
                vec![
                    vec![vec![vec![None; divide_num + 1]; divide_num + 1]; divide_num];
                    divide_num
                ];
                sample_color_num
            ];
        let similality_memo = vec![vec![vec![None; divide_num]; divide_num]; sample_color_num];
        DpAI {
            divide_num: divide_num,
            rng: rand::thread_rng(),
            sample_color_num,
            sampled_color: vec![],
            memo,
            memo_restore,
            similality_memo,
            image: image::Image::new(1, 1),
            initial_state: State::initial_state(0, 0, 0),
            initial_block: SimpleBlock {
                p: Point::new(0, 0),
                size: Point::new(0, 0),
                color: Color::ONE,
                state: BlockState::Active,
            },
        }
    }
    fn calc(&mut self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> i32 {
        let d = self.divide_num;
        assert!(x + w <= d);
        assert!(y + h <= d);
        {
            let memo_item = self.memo[color_id][x][y][w][h];
            if memo_item < (1 << 30) {
                return memo_item;
            }
        }
        let mut ret = (
            self.calc_similality(x, y, w, h, color_id),
            ArrayVec::<_, 2>::new(),
            ArrayVec::<_, 4>::new(),
        );
        let lt = self.convert_point(x, y);
        let rb = self.convert_point(x + w, y + h);
        let target_area = ((rb.x - lt.x) * (rb.y - lt.y)) as usize;
        assert!(target_area > 0);
        for c in 0..self.sampled_color.len() {
            let mut color_move = None;
            let mut ncost = 0;
            if c != color_id {
                // Color
                let mv = Move::Color {
                    block_id: BlockId::default(),
                    color: self.sampled_color[c],
                };
                ncost += simulator::move_cost_without_state(
                    &mv,
                    target_area,
                    self.image.width(),
                    self.image.height(),
                    self.initial_state.cost_coeff_version,
                ) as i32;
                let scost = self.calc_similality(x, y, w, h, c);
                if ncost + scost < ret.0 {
                    ret.0 = ncost + scost;
                    ret.1 = ArrayVec::new();
                    ret.1.push(mv.clone());
                    ret.2 = ArrayVec::new();
                }
                color_move = Some(mv);
            }
            // PCut
            for lw in 1..w {
                for lh in 1..h {
                    let mv = Move::PCut {
                        block_id: BlockId::default(),
                        point: self.convert_point(x + lw, y + lh),
                    };
                    let dx = [0, lw, lw, 0];
                    let dy = [0, 0, lh, lh];
                    let mut nlcost = simulator::move_cost_without_state(
                        &mv,
                        target_area,
                        self.image.width(),
                        self.image.height(),
                        self.initial_state.cost_coeff_version,
                    ) as i32;
                    let mut nlchilds = ArrayVec::<_, 4>::new();
                    for i in 0..4 {
                        let nx = x + dx[i];
                        let ny = y + dy[i];
                        let nw = [lw, w - lw, w - lw, lw][i];
                        let nh = [lh, lh, h - lh, h - lh][i];
                        let nret = self.calc(nx, ny, nw, nh, c);
                        nlcost += nret;
                        nlchilds.push(Child::new(nx, ny, nw, nh, c));
                    }
                    if ncost + nlcost < ret.0 {
                        ret.0 = ncost + nlcost;
                        ret.1 = ArrayVec::new();
                        if let Some(cmv) = color_move.clone() {
                            ret.1.push(cmv);
                        }
                        ret.1.push(mv);
                        ret.2 = nlchilds;
                    }
                }
            }
            // LCut
            for lw in 1..w {
                let mv = Move::LCut {
                    block_id: BlockId::default(),
                    orientation: Orientation::Vertical,
                    line_number: self.convert_point(x + lw, y).x,
                };
                let dx = [0, lw];
                let dy = [0, 0];
                let mut nlcost = simulator::move_cost_without_state(
                    &mv,
                    target_area,
                    self.image.width(),
                    self.image.height(),
                    self.initial_state.cost_coeff_version,
                ) as i32;
                let mut nlchilds = ArrayVec::<_, 4>::new();
                for i in 0..2 {
                    let nx = x + dx[i];
                    let ny = y + dy[i];
                    let nw = [lw, w - lw][i];
                    let nh = [h, h][i];
                    let nret = self.calc(nx, ny, nw, nh, c);
                    nlcost += nret;
                    nlchilds.push(Child::new(nx, ny, nw, nh, c));
                }
                if ncost + nlcost < ret.0 {
                    ret.0 = ncost + nlcost;
                    ret.1 = ArrayVec::new();
                    if let Some(cmv) = color_move.clone() {
                        ret.1.push(cmv);
                    }
                    ret.1.push(mv);
                    ret.2 = nlchilds;
                }
            }
            for lh in 1..h {
                let mv = Move::LCut {
                    block_id: BlockId::default(),
                    orientation: Orientation::Horizontal,
                    line_number: self.convert_point(x, y + lh).y,
                };
                let dx = [0, 0];
                let dy = [0, lh];
                let mut nlcost = simulator::move_cost_without_state(
                    &mv,
                    target_area,
                    self.image.width(),
                    self.image.height(),
                    self.initial_state.cost_coeff_version,
                ) as i32;
                let mut nlchilds = ArrayVec::<_, 4>::new();
                for i in 0..2 {
                    let nx = x + dx[i];
                    let ny = y + dy[i];
                    let nw = [w, w][i];
                    let nh = [lh, h - lh][i];
                    let nret = self.calc(nx, ny, nw, nh, c);
                    nlcost += nret;
                    nlchilds.push(Child::new(nx, ny, nw, nh, c));
                }
                if ncost + nlcost < ret.0 {
                    ret.0 = ncost + nlcost;
                    ret.1 = ArrayVec::new();
                    if let Some(cmv) = color_move.clone() {
                        ret.1.push(cmv);
                    }
                    ret.1.push(mv);
                    ret.2 = nlchilds;
                }
            }
        }
        self.memo[color_id][x][y][w][h] = ret.0;
        self.memo_restore[color_id][x][y][w][h] = Some((ret.1, ret.2));
        // println!(
        //     "{} {} {} {} {} {} {:?} {}",
        //     x, y, w, h, color_id, ret.0, state, ret.1
        // );
        return ret.0;
    }

    // calcでProgramを返すとコピーが二乗行われるので後から復元する
    fn restore_program(
        &self,
        program: &mut Program,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color_id: usize,
        block_id: &mut BlockId,
    ) {
        let (mut lprogram, childs) = self.memo_restore[color_id][x][y][w][h].clone().unwrap();
        for mv in lprogram.iter_mut() {
            mv.convert_block_id(&block_id);
            program.0.push(mv.clone());
        }
        for i in 0..childs.len() {
            let child = childs[i];
            block_id.0.push(i as u16);
            self.restore_program(
                program,
                child.x,
                child.y,
                child.w,
                child.h,
                child.color_id,
                block_id,
            );
            block_id.0.pop();
        }
    }

    fn calc_similality(&mut self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> i32 {
        let mut ret = 0;
        for dx in 0..w {
            let nx = x + dx;
            for dy in 0..h {
                let ny = y + dy;
                if let Some(s) = self.similality_memo[color_id][nx][ny] {
                    ret += s;
                } else {
                    let lt = self.convert_point(nx, ny);
                    let rb = self.convert_point(nx + 1, ny + 1);
                    let s = simulator::calc_partial_one_color_similarity(
                        lt,
                        Point::new(rb.x - lt.x, rb.y - lt.y),
                        self.sampled_color[color_id],
                        &self.image,
                    ) as i32;
                    self.similality_memo[color_id][nx][ny] = Some(s);
                    ret += s;
                }
            }
        }
        return ret;
    }
    fn convert_point(&self, x: usize, y: usize) -> Point {
        let d = self.divide_num;
        let l = std::cmp::min(
            self.topleft().x + self.width() as i32,
            self.topleft().x + (x * self.width() / d) as i32,
        );
        let t = std::cmp::min(
            self.topleft().y + self.height() as i32,
            self.topleft().y + (y * self.height() / d) as i32,
        );
        return Point::new(l, t);
    }
    fn topleft(&self) -> Point {
        self.initial_block.p
    }
    fn width(&self) -> usize {
        self.initial_block.size.x as usize
    }
    fn height(&self) -> usize {
        self.initial_block.size.y as usize
    }
}

#[test]
fn dp_ai_test() {
    let mut blocks = std::collections::HashMap::new();
    let block_id = BlockId(vec![0, 0, 0, 2]);
    let simpel_block = SimpleBlock::new(Point::new(1, 1), Point::new(3, 2), Color::ONE);
    blocks.insert(block_id, simpel_block);
    let state = State {
        blocks,
        next_global_id: 10,
        cost_coeff_version: 0,
    };
    let image = image::Image::from_string_array(&[
        "rr.....", "bbggg..", "bbggg..", "bbggg..", "bbggg..", "bbggg..", "bbggg..", "bbggg..",
        "bbggg..",
    ]);
    let mut dp_ai = DpAI::new(2, 3);

    let dp_program = dp_ai.solve(&image, &state);
    assert!(dp_ai.convert_point(0, 0) == Point::new(1, 1));
    assert!(dp_ai.convert_point(2, 2) == Point::new(4, 3));

    match simulator::calc_score(&dp_program, &image, &state) {
        Ok(_score) => {}
        Err(error) => {
            log::debug!("{}", error);
            assert!(false);
        }
    };
}
