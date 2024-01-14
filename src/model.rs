use rand::prelude::*;
use std::{fs::File, num, time};

pub const CHAR: i32 = 16;
pub const HARI_PER_FLOOR: i32 = 30; // 30%
pub const ITEM_PERCENT: i32 = 15;
pub const MUTEKI_TIME: i32 = 4000; // 4sec (length of MUTEKI bgm)
pub const HIGHSCORES: i32 = 10;

pub mod Field {
    pub const WID: i32 = 18; // フィールド幅（壁を含まない。セル数）
    pub const HEI: i32 = 30; // フィールド高さ（セル数）
    pub const LEFT: i32 = super::CHAR * 1; //
    pub const RIGHT: i32 = LEFT + (super::CHAR * WID); // Ruby版では-1している。理由不明
    pub const TOP: i32 = 0;
    pub const BOTTOM: i32 = TOP + (super::CHAR * HEI);
    pub const FLOORWID: i32 = 5; // 1個の床のセル数
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Chara {
    EMPTY,
    WALL,
    BLOCK,
    HARI,
    HITO,
    SAKEBI,
    STAR,
    PARA,
    OMORI,
    HITOPARA,
    HITOOMORI,
    HITODEAD,
    HITOWAVE,
    HITOMUTEKI,
}

#[derive(Debug, Clone, Copy)]
pub struct Floor {
    pub _type: Chara,
    pub x: i32,
    pub y: i32,
    pub broken: bool,
}

impl Floor {
    pub fn new(__type: Chara, _x: i32, _y: i32) -> Floor {
        Floor {
            _type: __type,
            x: _x,
            y: _y,
            broken: false,
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
    pub requested_musics: Vec<&'static str>,
    pub hito: Hito,
    pub floors: Vec<Floor>,
    pub isfloor: bool,
    pub data: [[Chara; Field::WID as usize]; Field::HEI as usize],
    pub time: u32,
    pub count: i32,
    pub fps: i32,
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

        let mut game = Game {
            rng: rng,
            is_over: false,
            rest_count: 0,
            success_count: 0,
            life: 100,
            requested_sounds: Vec::new(),
            requested_musics: Vec::new(),
            hito: Hito::new(),
            floors: Vec::new(),
            isfloor: false,
            data: [[Chara::EMPTY; Field::WID as usize]; Field::HEI as usize],
            time: 0,
            count: 0,
            fps: 0,
        };

        // 最初の床を生成
        game.generate_floor();

        game
    }

    pub fn generate_floor(&mut self) -> (i32, Chara) {
        // self.floors = Vec::new();
        // self.floors.push(Floor::new(
        //     Chara::BLOCK,
        //     (Field::WID - Field::FLOORWID) / 2,
        //     Field::HEI - 1,
        // ));

        let mut pos = self.rand(Field::WID + Field::FLOORWID) - Field::FLOORWID;
        if pos < 0 {
            pos = 0;
        }
        if pos > Field::WID - Field::FLOORWID {
            pos = Field::WID - Field::FLOORWID;
        }

        let _type;
        if self.rand(100) < HARI_PER_FLOOR {
            _type = Chara::HARI;
        } else {
            _type = Chara::BLOCK;
        }

        for i in 0..Field::FLOORWID {
            self.data[(Field::HEI - 1) as usize][(pos + i) as usize] = _type;
        }

        return (pos, _type);

        // self.floors.push(Floor::new(_type, pos, Field::HEI - 1));
    }

    pub fn update(&mut self, command: Command, dt: u32) {
        if self.is_over {
            return;
        }

        self.update_hito(command, dt);
        self.scroll(dt);

        if self.life <= 0 {
            self.is_over = true;
            self.requested_musics.push("halt");
            self.requested_sounds.push("gameover.wav");
            return;
        }

        match command {
            _ => {}
        }

        self.count_fps(dt);
    }

    pub fn update_hito(&mut self, command: Command, dt: u32) -> i32 {
        let mut ret: i32 = 0;

        if command == Command::Left {
            if self.hito.x > 0 && self.can_pass(self.hito.x - 1, self.hito.y) {
                self.hito.x -= 1;
            }
        } else if command == Command::Right {
            if self.hito.x < Field::WID - 1 && self.can_pass(self.hito.x + 1, self.hito.y) {
                self.hito.x += 1;
            }
        }

        return ret;
    }

    pub fn scroll(&mut self, dt: u32) -> bool {
        if !self.can_pass(self.hito.x, self.hito.y + 1) {
            return false;
        }
        for i in 0..(Field::HEI - 1) {
            self.data[i as usize] = self.data[(i + 1) as usize];
        }

        for i in 0..Field::WID {
            self.data[(Field::HEI - 1) as usize][i as usize] = Chara::EMPTY;
        }

        if self.isfloor {
            let (pos, _type) = self.generate_floor();
        }

        // invert @isfloor
        self.isfloor = !self.isfloor;

        if !self.can_pass(self.hito.x, self.hito.y + 1) {
            if self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::BLOCK {
                self.requested_sounds.push("foot.wav");
            }
        }

        return true;
    }

    pub fn can_pass(&self, x: i32, y: i32) -> bool {
        match self.data[y as usize][x as usize] {
            Chara::EMPTY | Chara::STAR | Chara::PARA | Chara::OMORI => true,
            _ => false,
        }
    }

    pub fn is_gauge_red(&self) -> bool {
        false
    }

    pub fn rand(&mut self, max: i32) -> i32 {
        self.rng.gen_range(0..max)
    }

    pub fn count_fps(&mut self, dt: u32) {
        self.time += dt;
        self.count += 1;
        if self.time >= 1000 {
            self.fps = self.count;
            self.time -= 1000;
            self.count = 0;
        }
    }
}
