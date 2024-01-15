use rand::prelude::*;
use std::time;

use self::field::{FLOORWID, HEI};

pub const CHAR: i32 = 16;
pub const HARI_PER_FLOOR: i32 = 30; // 30%
pub const ITEM_PERCENT: i32 = 15;
pub const MUTEKI_TIME: i32 = 4000; // 4sec (length of MUTEKI bgm)
pub const HIGHSCORES: i32 = 10;

pub mod field {
    pub const WID: i32 = 18; // フィールド幅（壁を含まない。セル数）
    pub const HEI: i32 = 30; // フィールド高さ（セル数）
    pub const LEFT: i32 = super::CHAR * 1; //
    pub const RIGHT: i32 = LEFT + (super::CHAR * WID) - 1; // -1している理由不明
    pub const TOP: i32 = 0;
    // pub const BOTTOM: i32 = TOP + (super::CHAR * HEI);
    pub const FLOORWID: i32 = 5; // 1個の床のセル数
}

pub mod wait {
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
                                    // pub const DEMO_TIME: i32 = 1000 * 60 * 3; // 5min
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
    pub hide: bool,
    pub muteki: bool,
    pub para: bool,
    pub omori: bool,
    pub flashing: bool,
    pub mutekistart: u32,
    pub walktimer: Timer,
    pub flashtimer: Timer,
    pub wavetimer: Timer,
    pub mutekiflashtimer: Timer,
    pub haribreaktimer: Timer,
}

impl Hito {
    pub fn new() -> Hito {
        Hito {
            x: field::WID / 2 - 1,
            y: field::HEI / 2,
            hitonum: 0,
            hide: false,
            muteki: false,
            para: false,
            omori: false,
            flashing: false,
            mutekistart: 0,
            walktimer: Timer::new(wait::WALK),
            flashtimer: Timer::new(wait::HITOFLASH),
            wavetimer: Timer::new(wait::HITOWAVE),
            mutekiflashtimer: Timer::new(wait::MUTEKIFLASH),
            haribreaktimer: Timer::new(wait::HARIBREAK),
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
    // WALL,
    BLOCK,
    HARI,
    // HITO,
    // SAKEBI,
    STAR,
    PARA,
    OMORI,
    // HITOPARA,
    // HITOOMORI,
    // HITODEAD,
    // HITOWAVE,
    // HITOMUTEKI,
}

pub enum EffectType {
    BREAK, // 無敵＆重りで床を破壊したときのエフェクト
    PANG,  // パラシュートで針の上に着地したときのエフェクト
           // PTS,   // 未使用。床を破壊したときに10pt加算する構想だった模様
}

pub struct Effect {
    pub x: i32,
    pub y: i32,
    pub _type: EffectType,
    pub timer: Timer,
    pub state: i32,
    pub dead: bool,
}

impl Effect {
    pub fn new(x: i32, y: i32, _type: EffectType, timer: Timer) -> Effect {
        Effect {
            x,
            y,
            _type,
            timer,
            state: 0,
            dead: false,
        }
    }
}

pub const STATES_BREAK: i32 = 3;
pub const STATES_PANG: i32 = 3;
// pub const STATES_PTS: i32 = 7;

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

    pub fn wait<F: FnMut()>(&mut self, dt: u32, mut callback: F) {
        self.wait += dt as i32;
        while self.wait >= self.waittime {
            self.wait -= self.waittime;
            callback();
            if self.wait < self.waittime {
                self.reset();
            }
        }
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
            damagetimer: Timer::new(wait::DAMAGE),
            flashtimer: Timer::new(wait::GAUGEFLASH),
            damaging: false,
            flashing: false,
            is_red: false,
        }
    }
}

pub struct System {
    pub time: u32,
    pub count: i32,
    pub fps: i32,
}

impl System {
    pub fn new() -> System {
        System {
            time: 0,
            count: 0,
            fps: 0,
        }
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

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub life: i32,
    pub requested_sounds: Vec<&'static str>,
    pub requested_musics: Vec<&'static str>,
    pub hito: Hito,
    pub isfloor: bool,
    pub data: [[Chara; field::WID as usize]; field::HEI as usize],
    pub effects: Vec<Effect>,
    pub score: i32,
    pub highscore: Vec<i32>,
    pub falltimer: Timer,
    pub gameovertimer: Timer,
    pub gauge: DamageGauge,
    pub now: u32,
    pub system: System,
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
            life: 100,
            requested_sounds: Vec::new(),
            requested_musics: Vec::new(),
            hito: Hito::new(),
            isfloor: false,
            data: [[Chara::EMPTY; field::WID as usize]; field::HEI as usize],
            effects: Vec::new(),
            score: 0,
            highscore: Vec::new(),
            falltimer: Timer::new(wait::FALL),
            gameovertimer: Timer::new(wait::GAMEOVER),
            gauge: DamageGauge::new(),
            now: 0,
            system: System::new(),
        };

        // 最初の床を生成
        game.generate_floor();

        game
    }

    pub fn generate_floor(&mut self) -> (i32, Chara) {
        let mut pos = self.rand(field::WID + field::FLOORWID) - field::FLOORWID;
        if pos < 0 {
            pos = 0;
        }
        if pos > field::WID - field::FLOORWID {
            pos = field::WID - field::FLOORWID;
        }

        let _type;
        if self.rand(100) <= HARI_PER_FLOOR {
            // randを<=で比較しているのはバグで、正しくは<だと思う
            _type = Chara::HARI;
        } else {
            _type = Chara::BLOCK;
        }

        for i in 0..field::FLOORWID {
            self.data[(field::HEI - 1) as usize][(pos + i) as usize] = _type;
        }

        return (pos, _type);
    }

    pub fn update(&mut self, command: Command, dt: u32) {
        self.now += dt;

        self.update_hito(command, dt);
        self.update_damage(dt);
        self.update_effects(dt);

        if self.is_over {
            self.gameovertimer.wait(dt, || {
                if !self.hito.hide {
                    self.hito.hide = true;
                    self.add_highscore();
                }
            });
            // wait!(self.gameovertimer, dt, {
            //     if !self.hito.hide {
            //         self.hito.hide = true;
            //         self.add_highscore();
            //     }
            // });
            return;
        }

        wait!(self.falltimer, dt, {
            self.scroll();
        });

        if self.life <= 0 {
            self.is_over = true;
            self.requested_musics.push("halt");
            self.requested_sounds.push("gameover.wav");
            self.hito.start_flashing();
            return;
        }

        self.system.count_fps(dt);
    }

    pub fn update_hito(&mut self, command: Command, dt: u32) {
        if !self.is_over {
            wait!(self.hito.walktimer, dt, {
                // move
                if command == Command::Left {
                    if self.hito.x > 0 && self.can_pass(self.hito.x - 1, self.hito.y) {
                        self.hito.x -= 1;
                    }
                } else if command == Command::Right {
                    if self.hito.x < field::WID - 1 && self.can_pass(self.hito.x + 1, self.hito.y) {
                        self.hito.x += 1;
                    }
                }
            });

            // get item
            match self.data[self.hito.y as usize][self.hito.x as usize] {
                Chara::STAR => {
                    self.data[self.hito.y as usize][self.hito.x as usize] = Chara::EMPTY;
                    self.hito.muteki = true;
                    self.hito.mutekistart = self.now;
                    self.requested_musics.push("pause");
                    self.requested_sounds.push("muteki.wav");
                }
                Chara::PARA => {
                    self.data[self.hito.y as usize][self.hito.x as usize] = Chara::EMPTY;
                    self.set_scroll_wait(wait::FALL_PARA);
                    self.hito.para = true;
                    self.hito.omori = false;
                    self.requested_sounds.push("getpara.wav");
                }
                Chara::OMORI => {
                    self.data[self.hito.y as usize][self.hito.x as usize] = Chara::EMPTY;
                    self.set_scroll_wait(wait::FALL_OMORI);
                    self.hito.omori = true;
                    self.hito.para = false;
                    self.requested_sounds.push("getomori.wav");
                }
                _ => {}
            }

            // stop omori
            if self.hito.omori
                && self.hito.muteki
                && ((self.now - self.hito.mutekistart) as f32 >= MUTEKI_TIME as f32 * 0.8)
            {
                self.hito.omori = false;
                self.set_scroll_wait(wait::FALL);
            }

            // stop muteki
            if self.hito.muteki && ((self.now - self.hito.mutekistart) as f32 >= MUTEKI_TIME as f32)
            {
                self.hito.muteki = false;
                self.hito.hitonum = 0;
                self.requested_musics.push("resume");
            }

            // stop para
            if self.hito.para
                && self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::HARI
                && !self.hito.muteki
            {
                self.hito.para = false;
                self.set_scroll_wait(wait::FALL);
                self.requested_sounds.push("spank.wav");
                self.effects.push(Effect::new(
                    self.hito.x,
                    self.hito.y,
                    EffectType::PANG,
                    Timer::new(150),
                ));
            }

            // break!
            if self.hito.omori && self.hito.muteki {
                if self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::BLOCK {
                    self.field_break(self.hito.x, self.hito.y + 1);
                } else if self.data[(self.hito.y + 1) as usize][self.hito.x as usize] == Chara::HARI
                {
                    wait!(self.hito.haribreaktimer, dt, {
                        self.field_break(self.hito.x, self.hito.y + 1);
                    });
                }
            }
        }

        if self.hito.flashing {
            wait!(self.hito.flashtimer, dt, {
                self.hito.hitonum = 1 - self.hito.hitonum; // 0:white 1:red
            });
        }

        if self.hito.muteki {
            wait!(self.hito.mutekiflashtimer, dt, {
                self.hito.hitonum += 1;
                if self.hito.hitonum > 6 {
                    self.hito.hitonum = 0;
                }
            });
        }
    }

    pub fn field_break(&mut self, x: i32, y: i32) {
        self.data[y as usize][x as usize] = Chara::EMPTY;
        self.effects
            .push(Effect::new(x, y, EffectType::BREAK, Timer::new(150)));
        self.requested_sounds.push("break.wav");
    }

    pub fn set_scroll_wait(&mut self, wait: i32) {
        self.falltimer.set_wait(wait);
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
                if !self.is_over {
                    self.requested_sounds.push("damage.wav");
                }
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

    pub fn update_effects(&mut self, dt: u32) {
        for effect in &mut self.effects {
            match effect._type {
                EffectType::BREAK => {
                    wait!(effect.timer, dt, {
                        effect.state += 1;
                        if effect.state >= STATES_BREAK {
                            effect.dead = true;
                        }
                    });
                }
                EffectType::PANG => {
                    wait!(effect.timer, dt, {
                        effect.state += 1;
                        if effect.state >= STATES_PANG {
                            effect.dead = true;
                        }
                    });
                } // EffectType::PTS => {
                  //     wait!(effect.timer, dt, {
                  //         effect.state += 1;
                  //         if effect.state >= STATES_PTS {
                  //             effect.dead = true;
                  //         }
                  //     });
                  // }
            }
        }
        self.effects.retain(|effect| !effect.dead);
    }

    pub fn scroll(&mut self) -> bool {
        if !self.can_pass(self.hito.x, self.hito.y + 1) {
            return false;
        }
        for i in 0..(field::HEI - 1) {
            self.data[i as usize] = self.data[(i + 1) as usize];
        }

        for i in 0..field::WID {
            self.data[(field::HEI - 1) as usize][i as usize] = Chara::EMPTY;
        }

        self.effects_scroll();

        // make new floor(if @isfloor) & item(if @isfloor&&rand)
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

    pub fn effects_scroll(&mut self) {
        for effect in &mut self.effects {
            effect.y -= 1;
            if effect.y == 0 {
                effect.dead = true;
            }
        }
    }

    pub fn can_pass(&self, x: i32, y: i32) -> bool {
        match self.data[y as usize][x as usize] {
            Chara::EMPTY | Chara::STAR | Chara::PARA | Chara::OMORI => true,
            _ => false,
        }
    }

    pub fn add_highscore(&mut self) {
        self.highscore.push(self.score);
        self.highscore.sort_by(|a, b| b.cmp(a));
        self.highscore = self
            .highscore
            .iter()
            .take(HIGHSCORES as usize)
            .map(|x| *x)
            .collect();
    }

    pub fn rand(&mut self, max: i32) -> i32 {
        self.rng.gen_range(0..max)
    }
}
