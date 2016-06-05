use std::time::{Duration, Instant};

use glorious::{Behavior, Renderer};

use common::{Message, State};
use unit::Unit;
use grid_manager::render_unit;

const MOVE_TILE_MS: u64 = 50;

#[derive(Debug)]
pub struct UnitMover {
    unit: Option<Unit>,
    origin: (u32, u32),
    path: Vec<(u32, u32)>,
    index: usize,
    start: Option<Instant>,
}

fn as_millis(dur: Duration) -> u64 {
    dur.as_secs() * 1_000 + (dur.subsec_nanos() / 1_000_000) as u64
}

impl UnitMover {
    #[inline]
    pub fn new(unit: Unit, origin: (u32, u32), path: Vec<(u32, u32)>) -> UnitMover {
        UnitMover {
            unit: Some(unit),
            origin: origin,
            path: path,
            index: 0,
            start: None,
        }
    }
}

impl<'a> Behavior<State<'a>> for UnitMover {
    type Message = Message;

    fn update(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        let now = Instant::now();
        let start = match self.start {
            None => {
                self.start = Some(now);
                now
            }
            Some(start) => start,
        };
        let elapsed = now.duration_since(start);
        let i = as_millis(elapsed) / MOVE_TILE_MS;

        if i >= self.path.len() as u64 {
            let unit = self.unit.take().expect("missing unit");
            let to = *self.path.last().unwrap_or(&self.origin);
            state.grid.add_unit(unit, to);
            state.pop_modal(queue);
            queue.push(Message::UnitMoved(self.origin, to));
        } else {
            self.index = i as usize;
        }
    }

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        if let Some(ref unit) = self.unit {
            let pos = self.path[self.index];
            let rect = state.tile_rect(pos);
            render_unit(unit, rect, true, state, renderer);
        }
    }
}
