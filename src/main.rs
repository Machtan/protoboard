#![feature(question_mark)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate lru_time_cache;
extern crate rand;

#[macro_use]
extern crate glorious;
extern crate sdl2;
extern crate sdl2_image;
extern crate sdl2_ttf;

use std::cmp;
use std::env;

use glorious::{BoxedInputMapper, Device, Game, ResourceManager};
use rand::Rng;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::Mouse;
use sdl2::render::BlendMode;
use sdl2_image::{INIT_JPG, INIT_PNG};

use resources::{ARCHER_PATH, FIRA_SANS_PATH, FIRA_SANS_BOLD_PATH, PROTECTOR_PATH, RACCOON_PATH,
                WARRIOR_PATH};
use common::{Config, State};
use faction::Faction;
use grid::{Grid, Terrain};
use grid_manager::GridManager;
use scene::Scene;
use turner::TurnManager;
use unit::{AttackType, UnitType};

mod attack_range;
mod common;
mod faction;
mod grid;
mod grid_manager;
mod menus;
mod resources;
mod scene;
mod target_selector;
mod turner;
mod unit;
mod unit_mover;

// TODO: It might be, that the renderer argument for `Behavior::render`
// should in fact be `&mut Renderer<'a>`, rather than `&mut Renderer`.

fn main() {
    use sdl2::event::Event::*;
    use common::Message::*;

    // Load settings

    const WINDOW_TITLE: &'static str = "Raccoon Squad";
    const N_COLS: u32 = 20;
    const N_ROWS: u32 = 20;
    const TILE_SIZE: (u32, u32) = (32, 32);
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
    let mut renderer = window.renderer().present_vsync().build().unwrap();
    renderer.set_blend_mode(BlendMode::Blend);
    let _ = renderer.set_logical_size(w, h);

    let device = Device::new(renderer);
    let renderer = device.create_renderer();
    let resources = ResourceManager::new(&device, &font_context);

    // Load units

    let warrior_texture = resources.texture(WARRIOR_PATH);
    let archer_texture = resources.texture(ARCHER_PATH);
    let protector_texture = resources.texture(PROTECTOR_PATH);
    let raccoon_texture = resources.texture(RACCOON_PATH);

    let warrior = UnitType {
        texture: warrior_texture,
        health: 5,
        attack: AttackType::Melee,
        damage: 2,
        movement: 6,
    };
    let archer = UnitType {
        texture: archer_texture,
        health: 4,
        attack: AttackType::Ranged { min: 2, max: 3 },
        damage: 3,
        movement: 4,
    };
    let protector = UnitType {
        texture: protector_texture,
        health: 8,
        attack: AttackType::Melee,
        damage: 2,
        movement: 5,
    };
    let raccoon = UnitType {
        texture: raccoon_texture,
        health: 21,
        attack: AttackType::Spear { range: 3 },
        damage: 5,
        movement: 4,
    };

    // Set up game state.

    let config = Config {};
    let mut rng = rand::thread_rng();

    let mut grid = Grid::new((N_COLS, N_ROWS), |(x, y)| {
        let dist = cmp::min(y, N_ROWS - 1 - y);
        match dist {
            5 if x % 3 < 2 => Terrain::Mountains,
            _ => {
                if rng.next_f32() < 0.2 {
                    Terrain::Woods
                } else {
                    Terrain::Grass
                }
            }
        }
    });

    let unit_types = &[warrior, archer, protector];
    for i in 0..N_COLS {
        let unit_type = if i == N_COLS / 2 - 1 {
            raccoon.clone()
        } else {
            let index = i as usize % unit_types.len();
            unit_types[index].clone()
        };
        grid.add_unit(unit_type.create(Faction::Blue, None), (i, N_ROWS - 3));
    }
    grid.add_unit(raccoon.create(Faction::Red, None), (N_COLS / 2 - 4, 2));
    grid.add_unit(raccoon.create(Faction::Red, None), (N_COLS / 2 - 2, 2));
    grid.add_unit(raccoon.create(Faction::Red, None), (N_COLS / 2, 2));
    grid.add_unit(raccoon.create(Faction::Red, None), (N_COLS / 2 + 2, 2));

    let health_label_font = resources.font(FIRA_SANS_BOLD_PATH, 16);

    let mut state = State::new(resources,
                               grid,
                               TILE_SIZE,
                               NUMBER_OF_ACTIONS,
                               &health_label_font,
                               config);

    // Prepare the scene

    let mut scene = Scene::new();

    scene.add(Box::new(GridManager::new((0, 0))));

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
    // TODO: Does this work in high-DPI mode?
    mapper.add(map_event!(
        MouseWheel { x, y, .. },
        MouseScroll(x, y)
    ));

    // Run the main loop.

    let event_pump = sdl_context.event_pump().unwrap();
    let mut game = Game::new(MAX_FPS, renderer, event_pump);

    game.run(&mut state, &mapper, &mut scene, |m| *m == Exit);
}
