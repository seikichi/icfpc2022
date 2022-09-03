use crate::ai::HeadAI;
use crate::image;
use crate::isl::*;
use crate::simulator;
use crate::simulator::State;

pub struct MergeAI {
    state: State,
}

impl HeadAI for MergeAI {
    fn solve(&mut self, _image: &image::Image, _initial_state: &simulator::State) -> Program {
        let mut ret = Program(vec![]);
        while self.active_block_num() > 1 {
            let mut target = None;
            for (id1, block1) in self.state.blocks.iter() {
                if !block1.active {
                    continue;
                }
                for (id2, block2) in self.state.blocks.iter() {
                    if !block2.active || id1 == id2 {
                        continue;
                    }
                    if let Some(_next_block) = simulator::merge_block(block1, block2) {
                        target = Some((id1.clone(), id2.clone()));
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
        return ret;
    }
}

impl MergeAI {
    #[allow(dead_code)]
    pub fn new(initial_state: &State) -> Self {
        MergeAI {
            state: initial_state.clone(),
        }
    }
    fn active_block_num(&self) -> u32 {
        let mut ret = 0;
        for block in self.state.blocks.values() {
            if block.active {
                ret += 1;
            }
        }
        return ret;
    }
    // 最終的に全部まとめ終わった時のblock_id
    #[allow(dead_code)]
    pub fn merged_block_id(&self) -> u32 {
        assert!(self.active_block_num() == 1);
        return self.state.next_global_id - 1;
    }
}
