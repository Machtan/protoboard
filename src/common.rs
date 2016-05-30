extern crate glorious;

use glorious::{Behavior, ResourceManager};


#[derive(Debug, Clone)]
pub enum Message {
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    Confirm,
    CursorConfirm(u32, u32),
    Cancel,
    CursorCancel(u32, u32),
    FinishTurn,
    LeftClickAt(i32, i32),
    RightClickAt(i32, i32),
}

#[derive(Debug)]
pub struct State {
    pub resources: ResourceManager,
    pub player_turn: u32,
    pub player_count: u32,
}

impl State {
    pub fn new() -> Self {
        State {
            resources: ResourceManager::new(),
            player_turn: 1,
            player_count: 1,
        }
    }
}

pub type GameObject = Box<Behavior<State=State, Message=Message>>;
