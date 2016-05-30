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

impl Message {
    pub fn try_clone(&self) -> Option<Message> {
        use self::Message::*;
        Some(match *self {
            MoveCursorUp => MoveCursorUp,
            MoveCursorDown => MoveCursorDown,
            MoveCursorLeft => MoveCursorLeft,
            MoveCursorRight => MoveCursorRight,
            MoveCursorTo(x, y) => MoveCursorTo(x, y),
            Confirm => Confirm,
            CursorConfirm(x, y) => CursorConfirm(x, y),
            Cancel => Cancel,
            CursorCancel(x, y) => CursorCancel(x, y),
            FinishTurn => FinishTurn,
            LeftClickAt(x, y) => LeftClickAt(x, y),
            RightClickAt(x, y) => RightClickAt(x, y),
            PushModal(..) => return None,
            PopModal => PopModal,
            BreakModal => BreakModal,
        })
    }
}

#[derive(Debug)]
pub struct State {
    pub resources: ResourceManager,
    pub player_turn: u32,
    pub player_count: u32,
}

impl State {
    #[inline]
    pub fn new() -> State {
        State {
            resources: ResourceManager::new(),
            player_turn: 1,
            player_count: 1,
        }
    }
}

impl Default for State {
    #[inline]
    fn default() -> State {
        State::new()
    }
}

pub trait BehaviorDebug: Behavior + Debug {}

impl<T> BehaviorDebug for T where T: Behavior + Debug {}

pub type GameObject = Box<BehaviorDebug<State = State, Message = Message>>;
