#![feature(question_mark)]

use std::rc::Rc;

extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate glorious;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

use glorious::{BoxedInputMapper, Game, ExitSignal, Renderer, ResourceManager};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::Mouse;
use sdl2_image::{INIT_PNG, INIT_JPG};

use common::{FIRA_SANS_PATH, MARKER_PATH, RACCOON_PATH, State};
use grid::Grid;
use cursor::Cursor;
use scene::Scene;
use unit::AttackType;

mod common;
mod scene;
mod unit;
mod grid;
mod cursor;
mod menus;

pub fn main() {
    env_logger::LogBuilder::new()
        .filter(Some("protoboard"), log::LogLevelFilter::Trace)
        .init()
        .unwrap();

    use sdl2::event::Event::*;
    use common::Message::*;

    const WINDOW_TITLE: &'static str = "Protect P R 0 T 0 B 0 A R D";
    const N_COLS: u32 = 20;
    const N_ROWS: u32 = 20;
    const CELL_SIZE: (u32, u32) = (32, 32);
    const MAX_FPS: u32 = 60;

    // Set up SDL2.

    let sdl_context = sdl2::init().expect("could not initialize SDL2");
    let video_subsystem = sdl_context.video().expect("could not initialize video subsystem");
    let _image_context = sdl2_image::init(INIT_PNG | INIT_JPG)
        .expect("could not initialize SDL2_image");
    let font_context = sdl2_ttf::init().expect("Font init");
    // let mut limiter = FrameLimiter::new(60);

    // TODO: Implement scaling, so we can support high-DPI monitors.
    let window = video_subsystem.window(WINDOW_TITLE, 640, 640)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let renderer = Renderer::new(window.renderer().build().unwrap());
    let resources = ResourceManager::new(renderer.clone(), Rc::new(font_context));

    // Set up game state.

    let state = State::new(resources);

    // Cause a few assets to be preloaded.

    state.resources.texture(MARKER_PATH);
    let raccoon_texture = state.resources.texture(RACCOON_PATH);
    state.resources.font(FIRA_SANS_PATH, 16);

    let mut scene = Scene::new();

    let mut grid = Grid::new(N_COLS, N_ROWS, CELL_SIZE);
    let unit = unit::Unit::new(raccoon_texture, AttackType::Melee);
    for i in 0..N_COLS {
        grid.add_unit(unit.clone(), i, 0);
        grid.add_unit(unit.clone(), i, N_ROWS - 1);
    }
    scene.add(Box::new(grid));

    let cursor = Cursor::new(0, 0, N_COLS, N_ROWS, CELL_SIZE);
    scene.add(Box::new(cursor));

    // Set up input handling.

    let mut mapper = BoxedInputMapper::new();

    mapper.add(map_key_pressed!(Keycode::Up, MoveCursorUp));
    mapper.add(map_key_pressed!(Keycode::Down, MoveCursorDown));
    mapper.add(map_key_pressed!(Keycode::Left, MoveCursorLeft));
    mapper.add(map_key_pressed!(Keycode::Right, MoveCursorRight));

    mapper.add(map_scan_pressed!(Scancode::W, MoveCursorUp));
    mapper.add(map_scan_pressed!(Scancode::S, MoveCursorDown));
    mapper.add(map_scan_pressed!(Scancode::A, MoveCursorLeft));
    mapper.add(map_scan_pressed!(Scancode::D, MoveCursorRight));

    mapper.add(map_scan_pressed!(Scancode::Z, Confirm));
    mapper.add(map_scan_pressed!(Scancode::X, Cancel));
    mapper.add(map_event!(
         MouseButtonDown { x, y, mouse_btn: Mouse::Left, .. },
         LeftClickAt(x, y)
    ));
    mapper.add(map_event!(
         MouseButtonDown { x, y, mouse_btn: Mouse::Right, .. },
         RightClickAt(x, y)
    ));

    // Run the main loop.

    let event_pump = sdl_context.event_pump().unwrap();
    let mut game = Game::new(MAX_FPS, renderer, event_pump);

    game.run(state, &mapper, &mut scene, |signal| {
        match signal {
            ExitSignal::ApplicationQuit => true,
            ExitSignal::EscapePressed => {
                info!("Escape exit signal sent!");
                false
            }
        }
    });
}
