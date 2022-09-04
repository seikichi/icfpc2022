use std::collections::HashMap;

use crate::ai::HeadAI;
use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::SimpleBlock;
use crate::simulator::State;
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
    rng: ThreadRng,
    sample_color_num: usize,
    sampled_color: Vec<Color>,
    // memo[x][y][w][h][color_id] -> (score, Some(今のブロックに対するProgram, 復元用の次の最適解))
    memo: Vec<Vec<Vec<Vec<Vec<(i64, Option<(Program, Vec<Child>)>)>>>>>,
    similality_memo: Vec<Vec<Vec<Option<i64>>>>,
    image: image::Image,
    initial_state: State,
}

impl HeadAI for DpAI {
    fn solve(&mut self, image: &image::Image, initial_state: &simulator::State) -> Program {
        let mut ret = Program(vec![]);
        let mut initial_block_id = initial_state.blocks.keys().next().unwrap().clone();
        self.initial_state = initial_state.clone();
        if initial_state.blocks.len() != 1 {
            let mut merge_ai = MergeAI::new();
            ret = merge_ai.solve(image, initial_state);
            initial_block_id = merge_ai.merged_block_id();
            simulator::simulate_all(&ret, &self.initial_state).unwrap();
        }
        // color sampling
        self.image = image.clone();
        self.sampled_color =
            image::k_means_color_sampling(image, self.sample_color_num - 1, 20, &mut self.rng);
        self.sampled_color.push(Color::ONE); // TODO blockの色にする
        self.sampled_color.reverse();

        // dp
        let d = self.memo.len();
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
                vec![
                    vec![vec![(1 << 30, None); sample_color_num]; divide_num + 1];
                    divide_num + 1
                ];
                divide_num
            ];
            divide_num
        ];
        let similality_memo = vec![vec![vec![None; sample_color_num]; divide_num]; divide_num];
        DpAI {
            rng: rand::thread_rng(),
            sample_color_num,
            sampled_color: vec![],
            memo,
            similality_memo,
            image: image::Image::new(1, 1),
            initial_state: State::initial_state(0, 0),
        }
    }
    fn calc(&mut self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> i64 {
        let d = self.memo.len();
        assert!(x + w <= d);
        assert!(y + h <= d);
        if self.memo[x][y][w][h][color_id].1.is_some() {
            return self.memo[x][y][w][h][color_id].0;
        }
        let state = self.make_state(x, y, w, h, color_id);
        let mut ret = (
            self.calc_similality(x, y, w, h, color_id),
            Program(vec![]),
            vec![],
        );
        for c in 0..self.sampled_color.len() {
            let mut nprogram = Program(vec![]);
            let mut ncost = 0;
            if c != color_id {
                // Color
                nprogram.0.push(Move::Color {
                    block_id: BlockId::new(&vec![]),
                    color: self.sampled_color[c],
                });
                ncost += simulator::move_cost(
                    &state,
                    &nprogram.0[0],
                    self.image.width(),
                    self.image.height(),
                )
                .unwrap();
                // let nstate = self.make_state(x, y, w, h, c);
                let scost = self.calc_similality(x, y, w, h, c);
                if ncost + scost < ret.0 {
                    assert!(nprogram.0.len() == 1);
                    ret.0 = ncost + scost;
                    ret.1 = nprogram.clone();
                    ret.2 = vec![];
                }
            }
            // PCut
            for lw in 1..w {
                for lh in 1..h {
                    nprogram.0.push(Move::PCut {
                        block_id: BlockId::new(&vec![]),
                        point: self.convert_point(x + lw, y + lh),
                    });
                    let dx = [0, lw, lw, 0];
                    let dy = [0, 0, lh, lh];
                    let mut nlcost = simulator::move_cost(
                        &state,
                        &nprogram.0[0],
                        self.image.width(),
                        self.image.height(),
                    )
                    .unwrap();
                    let mut nlchilds = vec![];
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
                        ret.1 = nprogram.clone();
                        ret.2 = nlchilds;
                    }
                    nprogram.0.pop().unwrap();
                }
            }
            // LCut
            for lw in 1..w {
                nprogram.0.push(Move::LCut {
                    block_id: BlockId::new(&vec![]),
                    orientation: Orientation::Vertical,
                    line_number: self.convert_point(x + lw, y).x,
                });
                let dx = [0, lw];
                let dy = [0, 0];
                let mut nlcost = simulator::move_cost(
                    &state,
                    &nprogram.0[0],
                    self.image.width(),
                    self.image.height(),
                )
                .unwrap();
                let mut nlchilds = vec![];
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
                    ret.1 = nprogram.clone();
                    ret.2 = nlchilds;
                }
                nprogram.0.pop().unwrap();
            }
            for lh in 1..h {
                nprogram.0.push(Move::LCut {
                    block_id: BlockId::new(&vec![]),
                    orientation: Orientation::Horizontal,
                    line_number: self.convert_point(x, y + lh).y,
                });
                let dx = [0, 0];
                let dy = [0, lh];
                let mut nlcost = simulator::move_cost(
                    &state,
                    &nprogram.0[0],
                    self.image.width(),
                    self.image.height(),
                )
                .unwrap();
                let mut nlchilds = vec![];
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
                    ret.1 = nprogram.clone();
                    ret.2 = nlchilds;
                }
                nprogram.0.pop().unwrap();
            }
        }
        self.memo[x][y][w][h][color_id].0 = ret.0;
        self.memo[x][y][w][h][color_id].1 = Some((ret.1, ret.2));
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
        let (mut lprogram, childs) = self.memo[x][y][w][h][color_id].1.clone().unwrap();
        for mv in lprogram.0.iter_mut() {
            mv.convert_block_id(&block_id);
            program.0.push(mv.clone());
        }
        for i in 0..childs.len() {
            let child = childs[i];
            block_id.0.push_back(i as u32);
            self.restore_program(
                program,
                child.x,
                child.y,
                child.w,
                child.h,
                child.color_id,
                block_id,
            );
            block_id.0.pop_back();
        }
    }

    fn calc_similality(&mut self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> i64 {
        let mut ret = 0;
        for dx in 0..w {
            let nx = x + dx;
            for dy in 0..h {
                let ny = y + dy;
                if let Some(s) = self.similality_memo[nx][ny][color_id] {
                    ret += s;
                } else {
                    let s = simulator::calc_partial_one_color_similarity(
                        self.convert_point(nx, ny),
                        self.convert_point(1, 1),
                        self.sampled_color[color_id],
                        &self.image,
                    );
                    self.similality_memo[nx][ny][color_id] = Some(s);
                    ret += s;
                }
            }
        }
        return ret;
    }
    fn make_state(&self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> State {
        let d = self.memo.len();
        let l = self.topleft().x as usize + x * (self.width() / d);
        let t = self.topleft().y as usize + y * (self.height() / d);
        let pw = std::cmp::min((x + w) * (self.width() / d), self.width()) - l;
        let ph = std::cmp::min((y + h) * (self.height() / d), self.height()) - t;
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId::new(&vec![]),
            SimpleBlock::new(
                Point::new(l as i32, t as i32),
                Point::new(pw as i32, ph as i32),
                self.sampled_color[color_id],
            ),
        );
        State {
            blocks,
            next_global_id: 1,
        }
    }
    fn convert_point(&self, x: usize, y: usize) -> Point {
        let d = self.memo.len();
        let l = self.topleft().x + (x * (self.width() / d)) as i32;
        let t = self.topleft().y + (y * (self.height() / d)) as i32;
        return Point::new(l, t);
    }
    // fn renumber_block_id(&self, sub_programs: &mut Vec<Program>) -> Program {
    //     let mut ret = Program(vec![]);
    //     for i in 0..sub_programs.len() {
    //         for j in 0..sub_programs[i].0.len() {
    //             match &mut sub_programs[i].0[j] {
    //                 Move::PCut { block_id, point: _ } => block_id.0.push_front(i as u32),
    //                 Move::LCut {
    //                     block_id,
    //                     orientation: _,
    //                     line_number: _,
    //                 } => block_id.0.push_front(i as u32),
    //                 Move::Color { block_id, color: _ } => block_id.0.push_front(i as u32),
    //                 _ => {
    //                     unimplemented!()
    //                 }
    //             }
    //         }
    //         ret.0.append(&mut sub_programs[i].0);
    //     }
    //     return ret;
    // }
    fn topleft(&self) -> Point {
        Point::new(0, 0)
    }
    fn width(&self) -> usize {
        self.image.width()
    }
    fn height(&self) -> usize {
        self.image.height()
    }
}
