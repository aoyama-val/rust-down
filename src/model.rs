use rand::prelude::*;
use std::{num, time};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub rest_count: i32,
    pub success_count: i32,
    pub requested_sounds: Vec<&'static str>,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        // let rng = StdRng::seed_from_u64(0);

        let game = Game {
            rng: rng,
            is_over: false,
            rest_count: 0,
            success_count: 0,
            requested_sounds: Vec::new(),
        };

        game
    }

    pub fn init(&mut self) {}

    pub fn update(&mut self, command: Command, dt: u32) {
        if self.is_over {
            return;
        }

        match command {
            _ => {}
        }
    }
}
