use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::sys::KeyCode;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::fs;
use Field;
mod model;
use crate::model::*;

pub const SCREEN_W: i32 = 640;
pub const SCREEN_H: i32 = 480;

struct Image<'a> {
    texture: Texture<'a>,
    #[allow(dead_code)]
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
    fonts: HashMap<String, sdl2::ttf::Font<'a, 'a>>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-down", SCREEN_W as u32, SCREEN_H as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let music = sdl2::mixer::Music::from_file("./resources/sound/dark3.it")?;

    let timer = sdl_context.timer()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas, &ttf_context);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();
    game.reset();

    println!("Keys:");
    println!("  Left, Right : Move player");

    let mut before;
    let mut now = timer.ticks();

    // music.play(-1)?;

    'running: loop {
        let mut command = Command::None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    if code == Keycode::Escape {
                        break 'running;
                    }
                    if game.is_over {
                        game = Game::new();
                        game.reset();
                    } else {
                        match code {
                            Keycode::Left => command = Command::Left,
                            Keycode::Right => command = Command::Right,
                            _ => {}
                        };
                    }
                }
                _ => {}
            }
        }
        before = now;
        now = timer.ticks();
        let dt = now - before;
        game.update(command, dt);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    #[allow(unused_variables)] canvas: &mut Canvas<Window>,
    ttf_context: &'a Sdl2TtfContext,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
        fonts: HashMap::new(),
    };

    let entries = fs::read_dir("resources/image").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bmp") {
            let temp_surface = sdl2::surface::Surface::load_bmp(&path).unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&temp_surface)
                .expect(&format!("cannot load image: {}", path_str));

            let basename = path.file_name().unwrap().to_str().unwrap();
            let image = Image::new(texture);
            resources.images.insert(basename.to_string(), image);
        }
    }

    let entries = fs::read_dir("./resources/sound").unwrap();
    for entry in entries {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".wav") {
            let chunk = mixer::Chunk::from_file(path_str)
                .expect(&format!("cannot load sound: {}", path_str));
            let basename = path.file_name().unwrap().to_str().unwrap();
            resources.chunks.insert(basename.to_string(), chunk);
        }
    }

    load_font(
        &mut resources,
        &ttf_context,
        "./resources/font/boxfont2.ttf",
        20,
        "boxfont",
    );

    resources
}

fn load_font<'a>(
    resources: &mut Resources<'a>,
    ttf_context: &'a Sdl2TtfContext,
    path_str: &str,
    point_size: u16,
    key: &str,
) {
    let font = ttf_context
        .load_font(path_str, point_size)
        .expect(&format!("cannot load font: {}", path_str));
    resources.fonts.insert(key.to_string(), font);
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let font = resources.fonts.get_mut("boxfont").unwrap();

    render_font(
        canvas,
        font,
        format!("FPS:{}", 120).to_string(),
        640 - 80,
        0,
        Color::RGB(127, 127, 127),
        false,
    );

    render_font(
        canvas,
        font,
        "SCORE RANKING".to_string(),
        Field::RIGHT + 32,
        2,
        Color::RGB(255, 255, 255),
        false,
    );

    render_font(
        canvas,
        font,
        "SCORE:".to_string(),
        Field::RIGHT + 32,
        300,
        Color::RGB(255, 255, 255),
        false,
    );

    render_font(
        canvas,
        font,
        "LIFE".to_string(),
        Field::RIGHT + 32,
        330,
        Color::RGB(255, 255, 255),
        false,
    );

    render_font(
        canvas,
        font,
        "Ver.1.0.0".to_string(),
        640 - 106,
        460,
        Color::RGB(127, 127, 127),
        false,
    );

    // render walls
    for i in 0..Field::HEI {
        let image = resources.images.get("wall.bmp").unwrap();
        canvas
            .copy(
                &image.texture,
                Rect::new(0, 0, image.w, image.h),
                Rect::new(Field::LEFT - CHAR, CHAR * i, image.w, image.h),
            )
            .unwrap();
        canvas
            .copy(
                &image.texture,
                Rect::new(0, 0, image.w, image.h),
                Rect::new(Field::RIGHT, CHAR * i, image.w, image.h),
            )
            .unwrap();
    }

    // render floors and items
    for y in 0..Field::HEI {
        for x in 0..Field::WID {
            match game.data[y as usize][x as usize] {
                Chara::BLOCK => {
                    render_chara(canvas, resources, x, y, "floor.bmp", 0);
                }
                Chara::HARI => {
                    render_chara(canvas, resources, x, y, "floor.bmp", 1);
                }
                Chara::STAR => {
                    render_chara(canvas, resources, x, y, "item.bmp", 0);
                }
                Chara::PARA => {
                    render_chara(canvas, resources, x, y, "item.bmp", 1);
                }
                Chara::OMORI => {
                    render_chara(canvas, resources, x, y, "item.bmp", 2);
                }
                _ => {}
            }
        }
    }

    // render hito
    let image = resources.images.get("hito.bmp").unwrap();
    canvas
        .copy(
            &image.texture,
            Rect::new(CHAR * game.hito.hitonum, 0, CHAR as u32, CHAR as u32),
            Rect::new(
                Field::LEFT + (game.hito.x + 1) * CHAR,
                Field::TOP + game.hito.y * CHAR,
                CHAR as u32,
                CHAR as u32,
            ),
        )
        .unwrap();

    // render gauge
    let color = if game.is_gauge_red() {
        Color::RGB(255, 0, 0)
    } else {
        Color::RGB(255, 255, 255)
    };
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(
        Field::RIGHT + 80,
        SCREEN_H / 10 * 7,
        (((SCREEN_W - (CHAR * Field::WID) - 108) * game.life) / 100) as u32,
        16,
    ))?;

    // render effects

    canvas.present();

    Ok(())
}

fn render_chara(
    canvas: &mut Canvas<Window>,
    resources: &mut Resources,
    x: i32,
    y: i32,
    image: &str,
    image_index: usize,
) {
    let image = resources.images.get(image).unwrap();
    canvas
        .copy(
            &image.texture,
            Rect::new(CHAR * image_index as i32, 0, CHAR as u32, CHAR as u32),
            Rect::new(CHAR * (x + 1), CHAR * y, CHAR as u32, CHAR as u32),
        )
        .unwrap();
}

fn render_font(
    canvas: &mut Canvas<Window>,
    font: &sdl2::ttf::Font,
    text: String,
    x: i32,
    y: i32,
    color: Color,
    center: bool,
) {
    let texture_creator = canvas.texture_creator();

    let surface = font.render(&text).blended(color).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let x: i32 = if center {
        x - texture.query().width as i32 / 2
    } else {
        x
    };
    canvas
        .copy(
            &texture,
            None,
            Rect::new(x, y, texture.query().width, texture.query().height),
        )
        .unwrap();
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}
