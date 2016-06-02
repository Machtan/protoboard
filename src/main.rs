#![feature(question_mark)]

use std::env;
use std::rc::Rc;

extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate glorious;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

use glorious::{BoxedInputMapper, Game, Renderer, ResourceManager};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::Mouse;
use sdl2_image::{INIT_PNG, INIT_JPG};

use resources::{FIRA_SANS_PATH, MARKER_PATH, WARRIOR_PATH, ARCHER_PATH, RACCOON_PATH};
use common::State;
use grid::Grid;
use grid_manager::GridManager;
use cursor::Cursor;
use scene::Scene;
use unit::{AttackType, UnitType};
use faction::Faction;
use turner::TurnManager;

mod common;
mod resources;
mod scene;
mod faction;
mod unit;
mod grid;
mod grid_manager;
mod cursor;
mod menus;
mod target_selector;
mod turner;

pub fn main() {
    use sdl2::event::Event::*;
    use common::Message::*;

    // Load settings

    const WINDOW_TITLE: &'static str = "Protect P R 0 T 0 B 0 A R D";
    const N_COLS: u32 = 20;
    const N_ROWS: u32 = 20;
    const CELL_SIZE: (u32, u32) = (32, 32);
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

    // Set up SDL2.

    let sdl_context = sdl2::init().expect("could not initialize SDL2");
    let video_subsystem = sdl_context.video().expect("could not initialize video subsystem");
    let _image_context = sdl2_image::init(INIT_PNG | INIT_JPG)
        .expect("could not initialize SDL2_image");
    let font_context = sdl2_ttf::init().expect("Font init");
    // let mut limiter = FrameLimiter::new(60);

    let window = video_subsystem.window(WINDOW_TITLE, 640, 640)
        .allow_highdpi()
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let (w, h) = window.size();
    let (pw, ph) = window.drawable_size();
    let mut renderer = window.renderer().build().unwrap();
    let _ = renderer.set_logical_size(w, h);

    let renderer = Renderer::new(renderer);
    let resources = ResourceManager::new(renderer.clone(), Rc::new(font_context));

    // Set up game state.

    let mut state = State::new(resources, NUMBER_OF_ACTIONS);

    // Cause a few assets to be preloaded.

    state.resources.texture(MARKER_PATH);
    state.resources.font(FIRA_SANS_PATH, 16);

    // Load units

    let warrior_texture = state.resources.texture(WARRIOR_PATH);
    let archer_texture = state.resources.texture(ARCHER_PATH);
    let raccoon_texture = state.resources.texture(RACCOON_PATH);
    let warrior = UnitType::new(warrior_texture, 5, AttackType::Melee, 2);
    let archer = UnitType::new(archer_texture, 5, AttackType::Ranged { min: 2, max: 3 }, 2);
    let raccoon = UnitType::new(raccoon_texture, 25, AttackType::Melee, 5);

    // Prepare the scene

    let mut scene = Scene::new();

    let mut grid = Grid::new((N_COLS, N_ROWS));

    for i in 0..N_COLS {
        let unit_type = if i == N_COLS / 2 - 1 {
            raccoon.clone()
        } else if i % 2 == 0 {
            warrior.clone()
        } else {
            archer.clone()
        };
        grid.add_unit(unit_type.create(Faction::Red, None), (i, 0));
        grid.add_unit(unit_type.create(Faction::Blue, None), (i, N_ROWS - 1));
    }
    scene.add(Box::new(GridManager::new(grid, CELL_SIZE)));

    let cursor = Cursor::new((0, 0), (N_COLS, N_ROWS), CELL_SIZE);
    scene.add(Box::new(cursor));

    let turner = TurnManager::new(NUMBER_OF_ACTIONS,
                                  vec![Faction::Red, Faction::Blue],
                                  state.resources.font(FIRA_SANS_PATH, 16),
                                  &state);
    scene.add(Box::new(turner));

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
         MouseButtonDown { x, y, mouse_btn: Mouse::Right, .. },
         RightClickAt((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));
    mapper.add(map_event!(
        MouseMotion { x, y, .. },
        MouseMovedTo((x * pw as i32) / w as i32, (y * ph as i32) / h as i32)
    ));

    // Run the main loop.

    let event_pump = sdl_context.event_pump().unwrap();
    let mut game = Game::new(MAX_FPS, renderer, event_pump);

    game.run(&mut state, &mapper, &mut scene, |m| *m == Exit);
}
