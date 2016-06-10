#![feature(question_mark)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
extern crate rand;
extern crate serde;
extern crate serde_json as json;

#[macro_use]
extern crate glorious;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;
extern crate toml;

extern crate spec;

use std::env;
use std::process;

use glorious::{BoxedInputMapper, Color, Device, Game, ResourceManager};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::Mouse;
use sdl2::render::BlendMode;
use sdl2_image::{INIT_JPG, INIT_PNG};

use common::{Config, State};
use faction::Faction;
use info::GameInfo;
use level::Level;
use load::load_toml;
use resources::FIRA_SANS_BOLD_PATH;
use scene::Scene;

mod common;
mod faction;
mod grid;
mod grid_manager;
mod info;
mod info_box;
mod level;
mod load;
mod menus;
mod range;
mod resources;
mod scene;
mod target_selector;
mod tile;
mod unit;
mod unit_mover;

fn main() {
    use sdl2::event::Event::*;
    use common::Message::*;

    // Load settings

    const WINDOW_TITLE: &'static str = "Raccoon Squad";
    const TILE_SIZE: (u32, u32) = (48, 48);
    const MAX_FPS: u32 = 60;
    const NUMBER_OF_ACTIONS: u32 = 4;

    // Set up logging.

    let mut builder = env_logger::LogBuilder::new();
    builder.format(|record| {
        format!("[{}][{}] {}",
                record.level(),
                record.location().module_path(),
                record.args())
    });

    // Set default level to debug.
    // (setting this before `parse`, makes it be considered *after* env vars (for now).)
    builder.filter(Some("protoboard"), log::LogLevelFilter::Debug);
    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse(&var);
    }
    builder.init().unwrap();

    // Load level

    let info = match load_toml("info.toml", |m| warn!("{}", m)) {
        Ok(spec) => GameInfo::from_spec(spec).expect("could not validate info file"),
        Err(err) => {
            error!("could not load info file: {}", err);
            process::exit(1);
        }
    };
    let level = match load::load_json("level.json") {
        Ok(spec) => Level::from_spec(spec).expect("could not validate level"),
        Err(err) => {
            error!("could not load level: {}", err);
            process::exit(1);
        }
    };
    let grid = level.create_grid(&info);

    // Set up SDL2.

    let sdl_context = sdl2::init().expect("could not initialize SDL2");
    let video_subsystem = sdl_context.video().expect("could not initialize video subsystem");
    let _image_context = sdl2_image::init(INIT_PNG | INIT_JPG)
        .expect("could not initialize SDL2_image");
    let font_context = sdl2_ttf::init().expect("Font init");
    // let mut limiter = FrameLimiter::new(60);

    let window = video_subsystem.window(WINDOW_TITLE, 1024, 704)
        .allow_highdpi()
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let (w, h) = window.size();
    let (pw, ph) = window.drawable_size();
    let mut renderer = window.renderer().present_vsync().build().unwrap();
    renderer.set_blend_mode(BlendMode::Blend);
    let _ = renderer.set_logical_size(w, h);

    let device = Device::new(renderer);
    let renderer = device.create_renderer();
    let resources = ResourceManager::new(&device, &font_context);

    // Set up game state.

    let config = Config {};

    let health_label_font = resources.font(FIRA_SANS_BOLD_PATH, 13);
    let mut state = State::new(resources,
                               grid,
                               TILE_SIZE,
                               vec![Faction::Red, Faction::Blue],
                               NUMBER_OF_ACTIONS,
                               &health_label_font,
                               config);

    // Prepare the scene

    let mut scene = Scene::new(&state);

    // Set up input handling.

    let mut mapper = BoxedInputMapper::new();

    mapper.add(map_event!(Quit { .. }, Exit));

    mapper.add(map_key_pressed!(Keycode::Up, MoveCursorUp));
    mapper.add(map_key_pressed!(Keycode::Down, MoveCursorDown));
    mapper.add(map_key_pressed!(Keycode::Left, MoveCursorLeft));
    mapper.add(map_key_pressed!(Keycode::Right, MoveCursorRight));

    mapper.add(map_scan_pressed!(Scancode::W, MoveCursorUp));
    mapper.add(map_scan_pressed!(Scancode::S, MoveCursorDown));
    mapper.add(map_scan_pressed!(Scancode::A, MoveCursorLeft));
    mapper.add(map_scan_pressed!(Scancode::D, MoveCursorRight));

    mapper.add(map_scan_pressed!(Scancode::Space, FinishTurn));
    mapper.add(map_scan_pressed!(Scancode::Z, Confirm));
    mapper.add(map_scan_pressed!(Scancode::X, Cancel));
    mapper.add(map_scan_released!(Scancode::X, CancelReleased));
    mapper.add(map_event!(
         MouseButtonDown { x, y, mouse_btn: Mouse::Left, .. },
         LeftClickAt((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
         MouseButtonUp { x, y, mouse_btn: Mouse::Left, .. },
         LeftReleasedAt((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
         MouseButtonDown { x, y, mouse_btn: Mouse::Right, .. },
         RightClickAt((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
         MouseButtonUp { x, y, mouse_btn: Mouse::Right, .. },
         RightReleasedAt((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
        MouseMotion { x, y, .. },
        MouseMovedTo((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
        MouseWheel { x, y, .. },
        MouseScroll(x, y)
    ));

    // Run the main loop.

    let event_pump = sdl_context.event_pump().unwrap();
    let mut game =
        Game::with_clear_color(Color(0x66, 0x66, 0x66, 0xff), MAX_FPS, renderer, event_pump);

    game.run(&mut state, &mapper, &mut scene, |m| *m == Exit);
}
