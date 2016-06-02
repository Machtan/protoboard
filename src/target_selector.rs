use glorious::{Behavior, Renderer, Sprite};

use common::{Message, State};
use resources::CROSSHAIR_PATH;

#[derive(Debug)]
pub struct TargetSelector {
    pos: (u32, u32),
    origin: (u32, u32),
    selected: usize,
    tile_size: (u32, u32),
    grid_size: (u32, u32),
    targets: Vec<(u32, u32)>,
}

impl TargetSelector {
    pub fn new(pos: (u32, u32),
               origin: (u32, u32),
               grid_size: (u32, u32),
               tile_size: (u32, u32),
               targets: Vec<(u32, u32)>)
               -> TargetSelector {
        assert!(!targets.is_empty(), "No targets given to selector");
        TargetSelector {
            pos: pos,
            origin: origin,
            selected: 0,
            grid_size: grid_size,
            tile_size: tile_size,
            targets: targets,
        }
    }

    fn confirm<'a>(&self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        use common::Message::*;
        let selected = self.targets[self.selected];
        info!("Attacking target at {:?}", selected);
        // TODO: It might be better to have a cleaner model for
        // breaking out of a given number of modals. We might
        // want to have non-menu modals not be broken here?
        state.break_modal(queue);
        queue.push(AttackWithUnit(self.pos, selected));
        queue.push(UnitSpent(self.pos));
        queue.push(Deselect);
        queue.push(ShowCursor);
    }

    fn cancel<'a>(&self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        use common::Message::*;
        state.break_modal(queue);
        queue.push(MoveUnit(self.pos, self.origin));
        queue.push(MoveUnitAndAct(self.origin, self.pos));
    }
}

impl<'a> Behavior<State<'a>> for TargetSelector {
    type Message = Message;

    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            Confirm => {
                self.confirm(state, queue);
            }
            Cancel => {
                self.cancel(state, queue);
            }
            MoveCursorDown | MoveCursorRight => {
                self.selected = (self.selected + 1) % self.targets.len();
            }
            MoveCursorUp | MoveCursorLeft => {
                self.selected = (self.selected + self.targets.len() - 1) % self.targets.len();
            }
            MouseMovedTo(x, y) |
            LeftClickAt(x, y) => {
                assert!(x >= 0 && y >= 0);
                let (w, h) = self.tile_size;
                let (_, rows) = self.grid_size;
                let col = (x as u32 - (x as u32 % w)) / w;
                let row = rows - 1 - (y as u32 - (y as u32 % h)) / h;

                let mut is_valid_target = false;
                for (i, &pos) in self.targets.iter().enumerate() {
                    if pos == (col, row) {
                        self.selected = i;
                        is_valid_target = true;
                    }
                }

                if let LeftClickAt(..) = message {
                    if is_valid_target {
                        self.confirm(state, queue);
                    } else {
                        self.cancel(state, queue);
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        let (col, row) = self.targets[self.selected];
        let x = (col * self.tile_size.0) as i32;
        let grid_height = self.grid_size.1 * self.tile_size.1;
        let y = (grid_height - self.tile_size.1 - (row * self.tile_size.1)) as i32;
        let sprite = Sprite::new(state.resources.texture(CROSSHAIR_PATH), None);
        sprite.render(renderer, x, y, Some(self.tile_size));
    }
}
