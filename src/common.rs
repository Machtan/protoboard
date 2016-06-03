use std::fmt::Debug;

use faction::Faction;
use glorious::{Behavior, ResourceManager};

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

    Deselect,
    UnitSpent((u32, u32)),
    MoveUnit((u32, u32), (u32, u32)),
    MoveUnitAndAct((u32, u32), (u32, u32)),
    AttackWithUnit((u32, u32), (u32, u32)),

    ApplyOneModal,

    AttackSelected((u32, u32), (u32, u32)),
    WaitSelected,
    CancelSelected((u32, u32), (u32, u32)),

    Exit,
}

#[derive(Debug)]
pub enum ModalMessage<'a> {
    Push(GameObject<'a>),
    Pop,
    Break,
}

#[derive(Debug)]
pub struct State<'a> {
    pub resources: ResourceManager<'a>,
    pub current_turn: Faction,
    pub actions_left: u32,
    modal_stack: Vec<ModalMessage<'a>>,
}

impl<'a> State<'a> {
    #[inline]
    pub fn new(resources: ResourceManager<'a>, actions_left: u32) -> State<'a> {
        State {
            resources: resources,
            current_turn: Faction::Red,
            actions_left: actions_left,
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
}

pub trait BehaviorDebug<S>: Behavior<S> + Debug {}

impl<T, S> BehaviorDebug<S> for T where T: Behavior<S> + Debug {}

pub type GameObject<'a> = Box<BehaviorDebug<State<'a>, Message = Message>>;
