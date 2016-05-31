use std::fmt::Debug;

use glorious::{Behavior, ResourceManager};

pub const FIRA_SANS_PATH: &'static str = "assets/fonts/FiraSans-Regular.ttf";
pub const MARKER_PATH: &'static str = "assets/marker.png";
pub const RACCOON_PATH: &'static str = "assets/raccoon.png";

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
    UnitSpent(u32, u32),
    MoveUnit((u32, u32), (u32, u32)),
    SelectUnit(u32, u32),
    Deselect,
}

#[derive(Debug)]
pub struct State<'a> {
    pub resources: ResourceManager<'a>,
    player_turn: u32,
    player_count: u32,
    modal_stack: Vec<Option<GameObject<'a>>>,
}

impl<'a> State<'a> {
    #[inline]
    pub fn new(resources: ResourceManager<'a>) -> State<'a> {
        State {
            resources: resources,
            player_turn: 1,
            player_count: 1,
            modal_stack: Vec::new(),
        }
    }

    pub fn push_modal(&mut self, behavior: GameObject<'a>) {
        self.modal_stack.push(Some(behavior));
    }

    pub fn pop_modal(&mut self) {
        if self.modal_stack.pop().is_none() {
            self.modal_stack.push(None);
        }
    }

    pub fn apply_modal_stack(&mut self, dst: &mut Vec<GameObject<'a>>) {
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

pub trait BehaviorDebug<S>: Behavior<S> + Debug {}

impl<T, S> BehaviorDebug<S> for T where T: Behavior<S> + Debug {}

pub type GameObject<'a> = Box<BehaviorDebug<State<'a>, Message = Message>>;
