use std::fmt::Debug;
use glorious::{Behavior, ResourceManager};

#[derive(Clone, Debug)]
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
    MenuSelect(&'static str),
}

#[derive(Debug)]
pub struct State {
    pub resources: ResourceManager,
    player_turn: u32,
    player_count: u32,
    modal_stack: Vec<Option<GameObject>>,
}

impl State {
    #[inline]
    pub fn new() -> State {
        State {
            resources: ResourceManager::new(),
            player_turn: 1,
            player_count: 1,
            modal_stack: Vec::new(),
        }
    }

    pub fn push_modal(&mut self, behavior: GameObject) {
        self.modal_stack.push(Some(behavior));
    }

    pub fn pop_modal(&mut self) {
        if self.modal_stack.pop().is_none() {
            self.modal_stack.push(None);
        }
    }

    pub fn apply_modal_stack(&mut self, dst: &mut Vec<GameObject>) {
        for modal in self.modal_stack.drain(..) {
            match modal {
                Some(modal) => {
                    debug!("Pushing modal state: {:?}", modal);
                    dst.push(modal);
                }
                None => {
                    let old = dst.pop();
                    match old {
                        Some(old) => {
                            debug!("Popped modal state: {:?}", old);
                        }
                        None => panic!("cannot pop from empty modal queue"),
                    }
                }
            }
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
