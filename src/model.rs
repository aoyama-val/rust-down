use rand::prelude::*;
use std::{num, time};

pub const CHAR: i32 = 16;

pub mod Field {
    pub const WID: i32 = 18;
    pub const HEI: i32 = 30;
    pub const LEFT: i32 = super::CHAR * 1;
    pub const RIGHT: i32 = LEFT + (super::CHAR * WID) - 1;
    pub const TOP: i32 = 0;
    pub const BOTTOM: i32 = TOP + (super::CHAR * HEI);
    pub const FLOORWID: i32 = 5;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
}

pub struct Hito {
    pub x: i32,
    pub y: i32,
    pub hitonum: i32,
}

impl Hito {
    pub fn new() -> Hito {
        Hito {
            x: Field::WID / 2 - 1,
            y: Field::HEI / 2,
            hitonum: 0,
        }
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub rest_count: i32,
    pub success_count: i32,
    pub life: i32,
    pub requested_sounds: Vec<&'static str>,
    pub hito: Hito,
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
            life: 0,
            requested_sounds: Vec::new(),
            hito: Hito::new(),
        };

        game
    }

    pub fn init(&mut self) {
        self.life = 100;
    }

    pub fn update(&mut self, command: Command, dt: u32) {
        if self.is_over {
            return;
        }

        match command {
            _ => {}
        }
    }

    pub fn is_gauge_red(&self) -> bool {
        false
    }
}
