use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::{
    image::Image,
    isl::{BlockId, Color, Point},
    simulator::{SimpleBlock, State},
};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct InitialBlock {
    blockId: String,
    bottomLeft: Vec<i32>,
    topRight: Vec<i32>,
    color: Vec<f32>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct InitialConfig {
    width: u32,
    height: u32,
    blocks: Vec<InitialBlock>,
}

fn load_inital_config(path: &str) -> Option<InitialConfig> {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_error) => return None,
    };
    // println!("{}", content);
    let config = serde_json::from_str(&content).unwrap();
    // println!("{:?}", config);
    return Some(config);
}

pub fn load_inistal_state(path: &str, image: &Image) -> State {
    if let Some(config) = load_inital_config(path) {
        let blocks = HashMap::new();
        let mut state = State {
            blocks,
            next_global_id: config.blocks.len() as u32,
        };
        assert!(config.width == image.width() as u32);
        assert!(config.height == image.height() as u32);
        for block in config.blocks.iter() {
            let p = Point::new(block.bottomLeft[0], block.bottomLeft[1]);
            let size = Point::new(block.topRight[0], block.topRight[1]) - p;
            let color = Color::new(
                block.color[0] / 255.0,
                block.color[1] / 255.0,
                block.color[2] / 255.0,
                block.color[3] / 255.0,
            );
            let simple_block = SimpleBlock::new(p, size, color);
            let block_id = vec![block.blockId.parse().expect("blockId is not integer")];
            state.blocks.insert(BlockId::new(&block_id), simple_block);
        }
        return state;
    } else {
        return State::initial_state(image.width() as i32, image.height() as i32);
    }
}
