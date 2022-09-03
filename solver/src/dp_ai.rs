use std::collections::HashMap;

use crate::ai::HeadAI;
use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::SimpleBlock;
use crate::simulator::State;
use rand::rngs::ThreadRng;
use rand::Rng;

pub struct DpAI {
    rng: ThreadRng,
    sampled_color: Vec<Color>,
    memo: Vec<Vec<Vec<Vec<Vec<Option<(i64, Program)>>>>>>,
    image: image::Image,
}

impl HeadAI for DpAI {
    fn solve(&mut self, image: &image::Image) -> Program {
        // color sampling
        self.image = image.clone();
        self.sampled_color[0] = Color::ONE;
        for i in 1..self.sampled_color.len() {
            // TODO 分散が大きくなるようにする
            let x = self.rng.gen_range(0..self.image.width());
            let y = self.rng.gen_range(0..self.image.height());
            self.sampled_color[i] = self.image.0[y][x];
        }
        // dp
        let d = self.memo.len();
        let (_score, program) = self.calc(0, 0, d, d, 0);
        let result = self.renumber_block_id(&mut vec![program]);
        // println!("{}", result);
        return result;
    }
}
impl DpAI {
    pub fn new(divide_num: usize, sample_color_num: usize) -> Self {
        let memo = vec![
            vec![
                vec![vec![vec![None; sample_color_num]; divide_num + 1]; divide_num + 1];
                divide_num
            ];
            divide_num
        ];
        DpAI {
            rng: rand::thread_rng(),
            sampled_color: vec![Color::ONE; sample_color_num],
            memo,
            image: image::Image::new(1, 1),
        }
    }
    fn calc(&mut self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> (i64, Program) {
        let d = self.memo.len();
        assert!(x + w <= d);
        assert!(y + h <= d);
        if let Some(ret) = self.memo[x][y][w][h][color_id].clone() {
            return ret;
        }
        let state = self.make_state(x, y, w, h, color_id);
        let mut ret = (
            simulator::calc_partial_state_similarity(
                self.convert_point(x, y),
                self.convert_point(w, h),
                &state,
                &self.image,
            ),
            Program(vec![]),
        );
        for c in 0..self.sampled_color.len() {
            let mut nprogram = Program(vec![]);
            let mut ncost = 0;
            if c != color_id {
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
                let nstate = self.make_state(x, y, w, h, c);
                let scost = simulator::calc_partial_state_similarity(
                    self.convert_point(x, y),
                    self.convert_point(w, h),
                    &nstate,
                    &self.image,
                );
                if ncost + scost < ret.0 {
                    // assert!(ncost + scost > 100);
                    assert!(nprogram.0.len() == 1);
                    ret.0 = ncost + scost;
                    ret.1 = nprogram.clone();
                }
            }
            for lw in 1..w {
                for lh in 1..h {
                    nprogram.0.push(Move::PCut {
                        block_id: BlockId::new(&vec![]),
                        point: self.convert_point(x + lw, y + lh),
                    });
                    let dx = [0, lw, lw, 0];
                    let dy = [0, 0, lh, lh];
                    let mut nlcost = 0;
                    let mut nlprogram = vec![];
                    for i in 0..4 {
                        let nx = x + dx[i];
                        let ny = y + dy[i];
                        let nw = [lw, w - lw, w - lw, lw][i];
                        let nh = [lh, lh, h - lh, h - lh][i];
                        let nret = self.calc(nx, ny, nw, nh, c);
                        nlcost += nret.0;
                        nlprogram.push(nret.1);
                    }
                    if ncost + nlcost < ret.0 {
                        ret.0 = ncost + nlcost;
                        // assert!(ncost + nlcost > 100);
                        ret.1 = nprogram.clone();
                        ret.1
                             .0
                            .append(&mut self.renumber_block_id(&mut nlprogram).0);
                    }
                    nprogram.0.pop().unwrap();
                }
            }
            // // TODO LCut
            // for lw in 1..w {
            //     //
            // }
            // for lh in 1..h {
            //     //
            // }
        }
        self.memo[x][y][w][h][color_id] = Some(ret.clone());
        // println!(
        //     "{} {} {} {} {} {} {:?} {}",
        //     x, y, w, h, color_id, ret.0, state, ret.1
        // );
        return ret;
    }

    fn make_state(&self, x: usize, y: usize, w: usize, h: usize, color_id: usize) -> State {
        let d = self.memo.len();
        let l = x * (self.image.width() / d);
        let t = y * (self.image.height() / d);
        let pw = std::cmp::min((x + w) * (self.image.width() / d), self.image.width()) - l;
        let ph = std::cmp::min((y + h) * (self.image.height() / d), self.image.height()) - t;
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
        let l = x * (self.image.width() / d);
        let t = y * (self.image.height() / d);
        return Point::new(l as i32, t as i32);
    }
    fn renumber_block_id(&self, sub_programs: &mut Vec<Program>) -> Program {
        let mut ret = Program(vec![]);
        for i in 0..sub_programs.len() {
            for j in 0..sub_programs[i].0.len() {
                match &mut sub_programs[i].0[j] {
                    Move::PCut { block_id, point: _ } => block_id.0.push_front(i as u32),
                    Move::LCut {
                        block_id,
                        orientation: _,
                        line_number: _,
                    } => block_id.0.push_front(i as u32),
                    Move::Color { block_id, color: _ } => block_id.0.push_front(i as u32),
                    _ => {
                        unimplemented!()
                    }
                }
            }
            ret.0.append(&mut sub_programs[i].0);
        }
        return ret;
    }
}
