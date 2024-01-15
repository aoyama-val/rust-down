use rand::prelude::*;
use std::{fs::File, num, time};

use self::Field::{FLOORWID, HEI};

pub const CHAR: i32 = 16;
pub const HARI_PER_FLOOR: i32 = 30; // 30%
pub const ITEM_PERCENT: i32 = 15;
pub const MUTEKI_TIME: i32 = 4000; // 4sec (length of MUTEKI bgm)
pub const HIGHSCORES: i32 = 10;

pub mod Field {
    pub const WID: i32 = 18; // フィールド幅（壁を含まない。セル数）
    pub const HEI: i32 = 30; // フィールド高さ（セル数）
    pub const LEFT: i32 = super::CHAR * 1; //
    pub const RIGHT: i32 = LEFT + (super::CHAR * WID) - 1; // -1している理由不明
    pub const TOP: i32 = 0;
    pub const BOTTOM: i32 = TOP + (super::CHAR * HEI);
    pub const FLOORWID: i32 = 5; // 1個の床のセル数
}

pub mod Wait {
    pub const FALL: i32 = 40;
    pub const FALL_PARA: i32 = 60;
    pub const FALL_OMORI: i32 = 20;
    pub const WALK: i32 = 83;
    pub const DAMAGE: i32 = 10; // ms/damage (life=100)
    pub const HITOFLASH: i32 = 80;
    pub const HITOWAVE: i32 = 200;
    pub const MUTEKIFLASH: i32 = 80;
    pub const GAUGEFLASH: i32 = 60;
    pub const HARIBREAK: i32 = 140;
    pub const GAMEOVER: i32 = 3400; // ms
    pub const DEMO_TIME: i32 = 1000 * 60 * 3; // 5min
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
    pub muteki: bool,
    pub omori: bool,
    pub flashing: bool,
    pub walktimer: Timer,
    pub flashtimer: Timer,
    pub wavetimer: Timer,
    pub mutekiflashtimer: Timer,
    pub haribreaktimer: Timer,
}

impl Hito {
    pub fn new() -> Hito {
        Hito {
            x: Field::WID / 2 - 1,
            y: Field::HEI / 2,
            hitonum: 0,
            muteki: false,
            omori: false,
            flashing: false,
            walktimer: Timer::new(Wait::WALK),
            flashtimer: Timer::new(Wait::HITOFLASH),
            wavetimer: Timer::new(Wait::HITOWAVE),
            mutekiflashtimer: Timer::new(Wait::MUTEKIFLASH),
            haribreaktimer: Timer::new(Wait::HARIBREAK),
        }
    }

    pub fn start_flashing(&mut self) {
        self.flashing = true;
    }

    pub fn stop_flashing(&mut self) {
        self.flashing = false;
        self.hitonum = 0;
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

pub struct Timer {
    waittime: i32,
    wait: i32,
}

impl Timer {
    pub fn new(waittime: i32) -> Timer {
        let mut timer = Timer {
            waittime: 0,
            wait: 0,
        };
        timer.set_wait(waittime);
        timer.reset();
        timer
    }

    pub fn reset(&mut self) {
        self.wait = 0;
    }

    pub fn add(&mut self, dt: u32) {
        self.wait += dt as i32;
    }

    pub fn is_reached(&mut self) -> bool {
        if self.wait >= self.waittime {
            self.wait -= self.waittime;
            if self.wait < self.waittime {
                self.reset();
            }
            return true;
        }
        return false;
    }

    pub fn set_wait(&mut self, t: i32) {
        assert!(t > 0);
        self.waittime = t;
    }
}

// Timerが規定値に達した場合に$blockを実行する
macro_rules! wait {
    ($timer_name:expr, $dt:ident, $block:block) => {
        $timer_name.add($dt);
        while $timer_name.is_reached() {
            $block
        }
    };
}

pub struct DamageGauge {
    pub damagetimer: Timer,
    pub flashtimer: Timer,
    pub damaging: bool,
    pub flashing: bool,
    pub is_red: bool,
}

impl DamageGauge {
    pub fn new() -> DamageGauge {
        DamageGauge {
            damagetimer: Timer::new(Wait::DAMAGE),
            flashtimer: Timer::new(Wait::GAUGEFLASH),
            damaging: false,
            flashing: false,
            is_red: false,
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
    pub score: i32,
    pub highscore: Vec<i32>,
    pub falltimer: Timer,
    pub gauge: DamageGauge,
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
            score: 0,
            highscore: Vec::new(),
            falltimer: Timer::new(Wait::FALL),
            gauge: DamageGauge::new(),
        };

        // 最初の床を生成
        game.generate_floor();

        game
    }

    pub fn generate_floor(&mut self) -> (i32, Chara) {
        let mut pos = self.rand(Field::WID + Field::FLOORWID) - Field::FLOORWID;
        if pos < 0 {
            pos = 0;
        }
        if pos > Field::WID - Field::FLOORWID {
            pos = Field::WID - Field::FLOORWID;
        }

        let _type;
        if self.rand(100) <= HARI_PER_FLOOR {
            // randを<=で比較しているのはバグで、正しくは<だと思う
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
        self.update_damage(dt);

        wait!(self.falltimer, dt, {
            self.scroll(dt);
        });

        if self.life <= 0 {
            self.is_over = true;
            self.requested_musics.push("halt");
            self.requested_sounds.push("gameover.wav");
            return;
        }

        self.count_fps(dt);
    }

    pub fn update_hito(&mut self, command: Command, dt: u32) {
        wait!(self.hito.walktimer, dt, {
            if command == Command::Left {
                if self.hito.x > 0 && self.can_pass(self.hito.x - 1, self.hito.y) {
                    self.hito.x -= 1;
                }
            } else if command == Command::Right {
                if self.hito.x < Field::WID - 1 && self.can_pass(self.hito.x + 1, self.hito.y) {
                    self.hito.x += 1;
                }
            }
        });

        if self.hito.flashing {
            wait!(self.hito.flashtimer, dt, {
                self.hito.hitonum = 1 - self.hito.hitonum; // 0:white 1:red
            });
        }
    }

    pub fn update_damage(&mut self, dt: u32) {
        if self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::HARI
            && !self.hito.muteki
        {
            // damage start
            if self.gauge.damaging == false {
                self.gauge.damaging = true;
                self.hito.start_flashing();
                self.gauge.flashing = true;
            }

            wait!(self.gauge.damagetimer, dt, {
                self.requested_sounds.push("damage.wav");
                self.life -= 1;
            });

            wait!(self.gauge.flashtimer, dt, {
                self.gauge.is_red = !self.gauge.is_red;
            });
        } else {
            // damage stop
            if self.gauge.damaging {
                self.gauge.damaging = false;
                self.hito.stop_flashing();
                self.gauge.flashing = false;
                self.gauge.is_red = false;
                self.gauge.damagetimer.reset();
            }
        }
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

            if self.rand(100) <= ITEM_PERCENT && _type != Chara::HARI {
                let r = self.rand(100);
                let item_type = if r <= 33 {
                    Chara::STAR
                } else if r <= 66 {
                    Chara::PARA
                } else {
                    Chara::OMORI
                };
                let x = pos + (FLOORWID / 2);
                let y = HEI - 2;
                self.data[y as usize][x as usize] = item_type;
            }
        }

        // invert @isfloor
        self.isfloor = !self.isfloor;

        if !self.can_pass(self.hito.x, self.hito.y + 1) {
            if self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::BLOCK {
                self.requested_sounds.push("foot.wav");
            }
        }

        self.score += 1;

        return true;
    }

    pub fn can_pass(&self, x: i32, y: i32) -> bool {
        match self.data[y as usize][x as usize] {
            Chara::EMPTY | Chara::STAR | Chara::PARA | Chara::OMORI => true,
            _ => false,
        }
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
