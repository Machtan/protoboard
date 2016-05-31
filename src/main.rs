#![feature(question_mark)]

#[macro_use]
extern crate glorious;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

use glorious::{BoxedInputMapper, Game, ExitSignal};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::Mouse;
use sdl2_image::{INIT_PNG, INIT_JPG};

use common::State;
use grid::Grid;
use cursor::Cursor;
use debug::DebugHelper;
use scene::Scene;

mod common;
mod scene;
mod unit;
mod grid;
mod cursor;
mod debug;
mod menus;

pub fn main() {
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

    let mut renderer = window.renderer().build().unwrap();

    // Set up game state.

    let mut state = State::new();
    // TODO: Consider when these functions can err.
    state.resources.load_texture("assets/marker.png", &mut renderer).unwrap();
    state.resources.load_texture("assets/raccoon.png", &mut renderer).unwrap();
    state.resources.create_sprite("marker", "assets/marker.png", None).unwrap();
    state.resources.create_sprite("raccoon", "assets/raccoon.png", None).unwrap();
    state.resources
        .load_font("firasans",
                   "assets/fonts/FiraSans-Regular.ttf",
                   16,
                   &font_context)
        .unwrap();
    state.resources
        .create_label("hello_world",
                      "firasans",
                      "Hello, World!",
                      (0, 0, 0, 0),
                      &mut renderer)
        .unwrap();

    let mut scene = Scene::new();

    let mut grid = Grid::new(N_COLS, N_ROWS, CELL_SIZE);
    let unit = unit::Unit::new("raccoon");
    for i in 0..N_COLS {
        // TODO: Consider when these functions can err.
        grid.add_unit(unit.clone(), i, 0).unwrap();
        grid.add_unit(unit.clone(), i, N_ROWS - 1).unwrap();
    }
    scene.add(Box::new(grid));

    let cursor = Cursor::new(0, 0, N_COLS, N_ROWS, CELL_SIZE);
    scene.add(Box::new(cursor));
    scene.add(Box::new(DebugHelper));

    let menu = menus::ModalMenu::new(&["Attack", "Wait"],
                                     0,
                                     (50, 50),
                                     "firasans",
                                     |_state, _message, _queue| {
                                         println!("Menu!");
                                     })
        .expect("could not create menu");
    scene.add(Box::new(menu));

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

    mapper.add(map_scan_pressed!(Scancode::X, Confirm));
    mapper.add(map_scan_pressed!(Scancode::Z, Cancel));
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
                println!("Escape exit signal sent!");
                false
            }
        }
    });
}
