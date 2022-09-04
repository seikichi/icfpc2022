use crate::ai::HeadAI;
use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::State;

pub struct MergeAI {
    state: State,
}

impl HeadAI for MergeAI {
    fn solve(&mut self, _image: &image::Image, initial_state: &simulator::State) -> Program {
        self.state = initial_state.clone();
        let mut ret = Program(vec![]);
        while self.active_block_num() > 1 {
            // 左上から順にマージする
            let blocks = self.state.blocks.clone();
            let mut blocks = blocks
                .iter()
                .filter(|(_key, value)| value.state.is_active())
                .map(|(key, value)| (key, value))
                .collect::<Vec<_>>();
            blocks.sort_by(|a, b| a.1.p.x.cmp(&b.1.p.x).then(a.1.p.y.cmp(&b.1.p.y)));
            let mut target = None;
            for i in 0..blocks.len() {
                for j in i + 1..blocks.len() {
                    if let Some(_next_block) = simulator::merge_block(&blocks[i].1, &blocks[j].1) {
                        target = Some((blocks[i].0.clone(), blocks[j].0.clone()));
                        break;
                    }
                }
                if target.is_some() {
                    break;
                }
            }
            if let Some((a, b)) = target {
                ret.0.push(Move::Merge { a, b });
                simulator::simulate(&mut self.state, ret.0.last().unwrap()).unwrap();
            } else {
                panic!("can't find mergable block");
            }
        }
        ret.0.push(Move::Color {
            block_id: self.merged_block_id(),
            color: Color::ONE,
        });
        return ret;
    }
}

impl MergeAI {
    #[allow(dead_code)]
    pub fn new() -> Self {
        MergeAI {
            state: State::initial_state(0, 0),
        }
    }
    fn active_block_num(&self) -> u32 {
        let mut ret = 0;
        for block in self.state.blocks.values() {
            if block.state.is_active() {
                ret += 1;
            }
        }
        return ret;
    }
    // 最終的に全部まとめ終わった時のblock_id
    #[allow(dead_code)]
    pub fn merged_block_id(&self) -> BlockId {
        assert!(self.active_block_num() == 1);
        return self
            .state
            .blocks
            .iter()
            .filter(|(_, value)| value.state.is_active())
            .next()
            .unwrap()
            .0
            .clone();
    }
}
