use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Duration;

use glorious::{Behavior, Color, Label, ResourceManager};
use lru_time_cache::LruCache;
use sdl2::rect::Rect;
use sdl2_ttf::Font;

use faction::Faction;
use grid::Grid;
use unit::Unit;

const COLOR_HEALTH_LABEL: Color = Color(0xff, 0xff, 0xff, 0xff);

pub trait DivFloor {
    fn div_floor(self, other: Self) -> Self;
}

impl DivFloor for i32 {
    #[inline]
    fn div_floor(self, other: i32) -> i32 {
        match (self / other, self % other) {
            (d, r) if (r < 0) != (other < 0) => d - 1,
            (d, _) => d,
        }
    }
}

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

    UnitSpent((u32, u32)),
    UnitMoved((u32, u32), (u32, u32)),
    AttackWithUnit((u32, u32), (u32, u32)),

    ApplyOneModal,

    AttackSelected((u32, u32), (u32, u32)),
    WaitSelected,
    CancelSelected((u32, u32), (u32, u32)),

    TargetSelectorCanceled((u32, u32), (u32, u32)),

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

    pub turn_info: TurnInfo,
    pub grid: Grid,
    pub window_size: (u32, u32),
    pub tile_size: (u32, u32),
    pub active_unit: Option<((u32, u32), Unit)>,
    pub camera_offset: (i32, i32),

    pub will_pop_modals: usize,
    modal_stack: Vec<ModalMessage<'a>>,

    pub health_label_font: &'a Font,
    health_labels: RefCell<LruCache<u32, Rc<Label>>>,
}

impl<'a> State<'a> {
    #[inline]
    pub fn new(resources: ResourceManager<'a, 'static>,
               grid: Grid,
               tile_size: (u32, u32),
               factions: Vec<Faction>,
               actions_left: u32,
               health_label_font: &'a Font,
               config: Config)
               -> State<'a> {
        let expiry_duration = Duration::from_millis(100);
        let window_size = resources.device().logical_size();
        let w = (window_size.0 / tile_size.0) as i32;
        let h = (window_size.1 / tile_size.1) as i32;
        let dx = (grid.size().0 as i32 - w).div_floor(2);
        let dy = (grid.size().1 as i32 - h).div_floor(2);
        State {
            config: config,
            resources: resources,
            turn_info: TurnInfo {
                factions: factions,
                current: 0,
                max_actions_left: actions_left,
                actions_left: actions_left,
            },
            grid: grid,
            window_size: window_size,
            tile_size: tile_size,
            active_unit: None,
            camera_offset: (dx, dy),
            health_label_font: health_label_font,
            will_pop_modals: 0,
            health_labels: RefCell::new(LruCache::with_expiry_duration(expiry_duration)),
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
        self.will_pop_modals += 1;
    }

    pub fn break_modal(&mut self, queue: &mut Vec<Message>) {
        self.modal_stack.push(ModalMessage::Break);
        queue.push(Message::ApplyOneModal);
        self.will_pop_modals += 1;
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
                self.will_pop_modals -= 1;
            }
            Break => {
                dst.clear();
                self.will_pop_modals -= 1;
            }
        }
    }

    pub fn window_to_grid(&self, x: i32, y: i32) -> Option<(u32, u32)> {
        let (tw, th) = self.tile_size;
        let h = self.window_size.1;

        let x = x.div_floor(tw as i32) + self.camera_offset.0;
        let y = h as i32 - y;
        let y = y.div_floor(th as i32) + self.camera_offset.1;

        let (w, h) = self.grid.size();
        if 0 <= x && x < w as i32 && 0 <= y && y < h as i32 {
            Some((x as u32, y as u32))
        } else {
            None
        }
    }

    pub fn tile_rect(&self, pos: (u32, u32)) -> Rect {
        let (tw, th) = self.tile_size;
        let h = self.window_size.1;

        let x = (pos.0 as i32 - self.camera_offset.0) * tw as i32;
        let y = (pos.1 as i32 - self.camera_offset.1) * th as i32;
        let y = h as i32 - y;

        Rect::new(x, y - th as i32, tw, th)
    }

    pub fn ensure_in_range(&mut self, pos: (u32, u32)) {
        let w = (self.window_size.0 / self.tile_size.0) as i32;
        let h = (self.window_size.1 / self.tile_size.1) as i32;
        let x = pos.0 as i32 - self.camera_offset.0;
        let y = pos.1 as i32 - self.camera_offset.1;

        if x < 1 {
            self.camera_offset.0 += x - 1;
        }
        if x > (w - 2) {
            self.camera_offset.0 += x - (w - 2);
        }
        if y < 1 {
            self.camera_offset.1 += y - 1;
        }
        if y > (h - 2) {
            self.camera_offset.1 += y - (h - 2);
        }
    }

    pub fn health_label(&self, health: u32) -> Rc<Label> {
        self.health_labels
            .borrow_mut()
            .entry(health)
            .or_insert_with(|| {
                let string = format!("{}", health);
                Rc::new(Label::new(self.health_label_font,
                                   &string,
                                   COLOR_HEALTH_LABEL,
                                   self.resources.device()))
            })
            .clone()
    }
}

#[derive(Clone, Debug)]
pub struct TurnInfo {
    factions: Vec<Faction>,
    current: usize,
    actions_left: u32,
    pub max_actions_left: u32,
}

impl TurnInfo {
    #[inline]
    pub fn end_turn(&mut self) {
        self.actions_left = self.max_actions_left;
        self.current = (self.current + 1) % self.factions.len();
    }

    #[inline]
    pub fn actions_left(&self) -> u32 {
        self.actions_left
    }

    #[inline]
    pub fn spend_action(&mut self) {
        assert!(self.actions_left > 0);
        self.actions_left = self.actions_left.saturating_sub(1);
    }

    #[inline]
    pub fn current_faction(&self) -> Faction {
        self.factions[self.current]
    }

    #[inline]
    pub fn remove_faction(&mut self, faction: Faction) {
        while let Some(i) = self.factions.iter().rposition(|&f| f == faction) {
            self.factions.remove(i);
            if self.current <= i {
                self.current = (self.current + self.factions.len() - 1) % self.factions.len();
            }
        }
    }

    #[inline]
    pub fn can_act(&self, unit: &Unit) -> bool {
        unit.faction == self.current_faction() && self.actions_left > 0 && !unit.spent
    }

    #[inline]
    pub fn factions(&self) -> &[Faction] {
        &self.factions
    }
}

pub struct Config {}

pub trait BehaviorDebug<S>: Behavior<S> + Debug {}

impl<T, S> BehaviorDebug<S> for T where T: Behavior<S> + Debug {}

pub type GameObject<'a> = Box<BehaviorDebug<State<'a>, Message = Message>>;
