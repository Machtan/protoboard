use std::time::{Duration, Instant};

use glorious::{Behavior, Renderer};

use common::{Message, State};
use unit::Unit;
use grid_manager::render_unit;

const MOVE_TILE_MS: u64 = 50;

#[derive(Debug)]
pub struct UnitMover {
    unit: Option<Unit>,
    from: (u32, u32),
    to: (u32, u32),
    start: Option<Instant>,
}

fn as_millis(dur: Duration) -> u64 {
    dur.as_secs() * 1000 + (dur.subsec_nanos() / 1_000_000) as u64
}

impl UnitMover {
    #[inline]
    pub fn new(unit: Unit, from: (u32, u32), to: (u32, u32)) -> UnitMover {
        UnitMover {
            unit: Some(unit),
            from: from,
            to: to,
            start: None,
        }
    }

    fn distance(&self) -> f32 {
        let dx = self.from.0 as f32 - self.to.0 as f32;
        let dy = self.from.1 as f32 - self.to.1 as f32;
        dx.hypot(dy)
    }

    fn lerp(&self) -> (u32, u32) {
        let elapsed = self.start.expect("render called before update").elapsed();
        let ems = as_millis(elapsed) as f32;
        let tms = MOVE_TILE_MS as f32 * self.distance();
        let t = ems / tms;

        let x = self.from.0 as f32 + (self.to.0 as f32 - self.from.0 as f32) * t;
        let y = self.from.1 as f32 + (self.to.1 as f32 - self.from.1 as f32) * t;
        (x.round() as u32, y.round() as u32)
    }
}

impl<'a> Behavior<State<'a>> for UnitMover {
    type Message = Message;

    fn update(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        match self.start {
            None => {
                self.start = Some(Instant::now());
            }
            Some(start) => {
                let elapsed = start.elapsed();
                if as_millis(elapsed) >= (MOVE_TILE_MS as f32 * self.distance()) as u64 {
                    let unit = self.unit.take().expect("missing unit");
                    state.grid.add_unit(unit, self.to);
                    state.pop_modal(queue);
                    queue.push(Message::UnitMoved(self.from, self.to));
                }
            }
        }
    }

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        if let Some(ref unit) = self.unit {
            let pos = self.lerp();
            let rect = state.tile_rect(pos);
            render_unit(unit, rect, true, state, renderer);
        }
    }
}
