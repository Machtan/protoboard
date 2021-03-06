use glorious::{Behavior, Renderer, Sprite};

use common::{Message, State};
use resources::CROSSHAIR_PATH;

#[derive(Debug)]
pub struct TargetSelector {
    pos: (u32, u32),
    origin: (u32, u32),
    selected: usize,
    targets: Vec<(u32, u32)>,
}

impl TargetSelector {
    pub fn new(pos: (u32, u32), origin: (u32, u32), targets: Vec<(u32, u32)>) -> TargetSelector {
        assert!(!targets.is_empty(), "No targets given to selector");
        TargetSelector {
            pos: pos,
            origin: origin,
            selected: 0,
            targets: targets,
        }
    }

    fn confirm(&self, state: &mut State, queue: &mut Vec<Message>) {
        use common::Message::*;

        let selected = self.targets[self.selected];
        debug!("Attacking target at {:?}", selected);
        state.break_modal(queue);
        queue.push(TargetConfirmed(self.pos, selected));
        queue.push(UnitSpent(self.pos));
    }

    fn cancel(&self, state: &mut State, queue: &mut Vec<Message>) {
        use common::Message::*;

        state.break_modal(queue);
        queue.push(TargetSelectorCanceled(self.origin, self.pos));
    }
}

impl<'a> Behavior<State<'a>> for TargetSelector {
    type Message = Message;

    fn update(&mut self, state: &mut State<'a>, _queue: &mut Vec<Message>) {
        state.ensure_in_range(self.targets[self.selected]);
    }

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
                let pos = match state.window_to_grid(x, y) {
                    Some(pos) => pos,
                    None => return,
                };

                let mut is_valid_target = false;
                for (i, &target) in self.targets.iter().enumerate() {
                    if pos == target {
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
        let rect = state.tile_rect(self.targets[self.selected]);
        let sprite = Sprite::new(state.resources.texture(CROSSHAIR_PATH), None);
        sprite.render_rect(renderer, rect);
    }
}
