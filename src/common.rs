
use std::fmt::{self, Debug};
use glorious::{Behavior, ResourceManager};


#[derive(Debug)]
pub enum Message {
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorTo(u32, u32),
    Confirm,
    CursorConfirm(u32, u32),
    Cancel,
    CursorCancel(u32, u32),
    FinishTurn,
    LeftClickAt(i32, i32),
    RightClickAt(i32, i32),
    PushModal(GameObject),
    PopModal,
    BreakModal,
}

#[derive(Debug)]
pub struct State {
    pub resources: ResourceManager,
    pub player_turn: u32,
    pub player_count: u32,
}

impl State {
    pub fn new() -> State {
        State {
            resources: ResourceManager::new(),
            player_turn: 1,
            player_count: 1,
        }
    }
}

pub trait DebugBehavior: Behavior + Debug {}

impl<T> DebugBehavior for T where T: Behavior + Debug {}

pub type GameObject = Box<DebugBehavior<State=State, Message=Message>>;
