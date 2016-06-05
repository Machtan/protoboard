use std::fmt::Debug;

use glorious::{Behavior, ResourceManager};
use sdl2::rect::Rect;

use faction::Faction;
use grid::Grid;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,

    Confirm,
    Cancel,
    CancelReleased,

    FinishTurn,
    FactionDefeated(Faction),
    FactionWins(Faction),

    LeftClickAt(i32, i32),
    LeftReleasedAt(i32, i32),
    RightClickAt(i32, i32),
    RightReleasedAt(i32, i32),
    MouseMovedTo(i32, i32),
    MouseScroll(i32, i32),

    Deselect,
    UnitSpent((u32, u32)),
    AttackWithUnit((u32, u32), (u32, u32)),

    ApplyOneModal,

    AttackSelected((u32, u32), (u32, u32)),
    WaitSelected,
    CancelSelected((u32, u32), (u32, u32)),

    TargetSelectorCanceled((u32, u32)),

    Exit,
}

#[derive(Debug)]
pub enum ModalMessage<'a> {
    Push(GameObject<'a>),
    Pop,
    Break,
}

pub struct State<'a> {
    pub config: Config,
    pub resources: ResourceManager<'a, 'static>,
    pub current_turn: Faction,
    pub actions_left: u32,
    pub grid: Grid,
    pub tile_size: (u32, u32),
    modal_stack: Vec<ModalMessage<'a>>,
}

impl<'a> State<'a> {
    #[inline]
    pub fn new(resources: ResourceManager<'a, 'static>,
               grid: Grid,
               tile_size: (u32, u32),
               actions_left: u32,
               config: Config)
               -> State<'a> {
        State {
            config: config,
            resources: resources,
            current_turn: Faction::Red,
            actions_left: actions_left,
            grid: grid,
            tile_size: tile_size,
            modal_stack: Vec::new(),
        }
    }

    pub fn push_modal(&mut self, behavior: GameObject<'a>, queue: &mut Vec<Message>) {
        self.modal_stack.push(ModalMessage::Push(behavior));
        queue.push(Message::ApplyOneModal);
    }

    pub fn pop_modal(&mut self, queue: &mut Vec<Message>) {
        self.modal_stack.push(ModalMessage::Pop);
        queue.push(Message::ApplyOneModal);
    }

    pub fn break_modal(&mut self, queue: &mut Vec<Message>) {
        self.modal_stack.push(ModalMessage::Break);
        queue.push(Message::ApplyOneModal);
    }

    pub fn apply_one_modal(&mut self, dst: &mut Vec<GameObject<'a>>) {
        use self::ModalMessage::*;
        let modal = self.modal_stack
            .pop()
            .expect("cannot apply modal message from empty stack");
        match modal {
            Push(modal) => {
                trace!("Pushing modal state: {:?}", modal);
                dst.push(modal);
            }
            Pop => {
                let old = dst.pop().expect("cannot pop modal from empty queue");
                trace!("Popped modal state: {:?}", old);
            }
            Break => {
                dst.clear();
            }
        }
    }

    pub fn window_to_grid(&self, x: i32, y: i32) -> (u32, u32) {
        assert!(x >= 0 && y >= 0);
        let (tw, th) = self.tile_size;
        let (_, h) = self.grid.size();
        let x = x as u32 / tw;
        let y = h - 1 - (y as u32) / th;
        (x, y)
    }

    pub fn tile_rect(&self, pos: (u32, u32)) -> Rect {
        let (tw, th) = self.tile_size;
        let (_, h) = self.grid.size();
        let x = pos.0 * tw;
        let y = h * th - th - (pos.1 * th);
        Rect::new(x as i32, y as i32, tw, th)
    }
}

pub struct Config {
    pub debug_movement: bool,
}

pub trait BehaviorDebug<S>: Behavior<S> + Debug {}

impl<T, S> BehaviorDebug<S> for T where T: Behavior<S> + Debug {}

pub type GameObject<'a> = Box<BehaviorDebug<State<'a>, Message = Message>>;
