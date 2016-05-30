#![allow(unused)]
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

#[macro_use]
extern crate glorious;

mod common;
mod unit;
mod grid;
mod cursor;
mod debug;

use std::rc::Rc;
use std::path::Path;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::event::Event;
use sdl2::mouse::Mouse;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;
use sdl2_ttf::Font;
use sdl2_image::{LoadTexture, INIT_PNG, INIT_JPG};

use glorious::{Game, Behavior, Sprite, ExitSignal};
use glorious::{BoxedInputMapper, FrameLimiter};

use common::{GameObject, State, Message};
use grid::Grid;
use cursor::Cursor;
use debug::DebugHelper;

struct Scene {
    objects: Vec<GameObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { objects: Vec::new() }
    }
}

impl Behavior for Scene {
    type State = State;
    type Message = Message;
    fn handle(&mut self, state: &mut Self::State, messages: &[Self::Message], 
            new_messages: &mut Vec<Self::Message>) {
        use common::Message::*;
        for message in messages.iter() {
            match message {
                _ => {}
            }
        }
        for object in &mut self.objects {
            object.handle(state, messages, new_messages);
        }
    }
    
    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        for object in &self.objects {
            object.render(state, renderer);
        }
    }
}

pub fn main() {
    // SDL2 SETUP
    use sdl2::event::Event::*;
    use common::Message::*;
    let sdl_context = sdl2::init().expect("Sdl init");
    let video_subsystem = sdl_context.video().expect("Video init");
    let _image_context = sdl2_image::init(INIT_PNG | INIT_JPG)
        .expect("Image init");
    let font_context = sdl2_ttf::init().expect("Font init");
    let mut limiter = FrameLimiter::new(60);
    
    const WINDOW_TITLE: &'static str = "Protect P R 0 T 0 B 0 A R D";
    let window = video_subsystem.window(WINDOW_TITLE, 640, 640)
        .position_centered()
        //.allow_highdpi() // Requires work on consistent scaling
        .opengl()
        .build()
        .unwrap();
    
    let mut renderer = window.renderer().build().unwrap();
    
    // GAME SETUP
    let mut state = State::new();
    state.resources.load_texture("assets/marker.png", &mut renderer);
    state.resources.load_texture("assets/raccoon.png", &mut renderer);
    state.resources.create_sprite("marker", "assets/marker.png", None);
    state.resources.create_sprite("raccoon", "assets/raccoon.png", None);
    state.resources.load_font("firasans", "assets/fonts/FiraSans-Regular.ttf",
        16, &font_context);
    state.resources.create_label("hello_world", "firasans", "Hello, World!", 
        (0, 0, 0, 0), &mut renderer);
    
    let mut scene = Scene::new();
    
    const N_COLS: u32 = 20;
    const N_ROWS: u32 = 20;
    const CELL_SIZE: (u32, u32) = (32, 32);
    
    let mut grid = Grid::new(N_COLS, N_ROWS, CELL_SIZE);
    let unit = unit::Unit::new("raccoon");
    for i in 0..N_COLS {
        grid.add_unit(unit.clone(), i, 0);
        grid.add_unit(unit.clone(), i, N_ROWS-1);
    }
    scene.objects.push(Box::new(grid));
    
    let cursor = Cursor::new(0, 0, N_COLS, N_ROWS, CELL_SIZE);
    scene.objects.push(Box::new(cursor));
    scene.objects.push(Box::new(DebugHelper));
    
    // INPUT SETUP
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
         MouseButtonDown { x, y, mouse_btn: Mouse::Left, ..},
         LeftClickAt(x, y)
    ));
    mapper.add(map_event!(
         MouseButtonDown { x, y, mouse_btn: Mouse::Right, ..},
         RightClickAt(x, y)
    ));
    
    // GO!
    let event_pump = sdl_context.event_pump().unwrap();
    const MAX_FPS: u32 = 60;
    let mut game = Game::new(
        MAX_FPS, renderer, event_pump
    );
    
    game.run(state, &mapper, &mut scene, |signal| {
        match signal {
            ExitSignal::ApplicationQuit => {
                true
            }
            ExitSignal::EscapePressed => {
                println!("Escape exit signal sent!");
                false
            }
        }
    });
}

