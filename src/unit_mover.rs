use std::time::{Duration, Instant};

use glorious::{Behavior, Renderer};
use sdl2::rect::Rect;

use common::{Message, State};
use unit::Unit;
use grid_manager::render_unit;

const MOVE_TILE_MS: u64 = 30;

#[derive(Debug)]
pub struct UnitMover {
    unit: Option<Unit>,
    origin: (u32, u32),
    path: Vec<(u32, u32)>,
    index: usize,
    delta: f32,
    start: Option<Instant>,
}

#[inline]
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
            delta: 0.0,
            start: None,
        }
    }

    fn current(&self) -> ((u32, u32), (u32, u32)) {
        if self.path.is_empty() {
            return (self.origin, self.origin);
        }
        if self.index == 0 {
            (self.origin, self.path[0])
        } else {
            (self.path[self.index - 1], self.path[self.index])
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
        let ms = as_millis(elapsed);
        let i = ms / MOVE_TILE_MS;

        if i >= self.path.len() as u64 {
            let unit = self.unit.take().expect("missing unit");
            let to = *self.path.last().unwrap_or(&self.origin);
            state.active_unit = Some((to, unit));
            state.pop_modal(queue);
            queue.push(Message::UnitMoved(self.origin, to));
        } else {
            self.index = i as usize;
            self.delta = (ms % MOVE_TILE_MS) as f32 / MOVE_TILE_MS as f32;
        }
    }

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        if let Some(ref unit) = self.unit {
            let (from, to) = self.current();
            let rect_a = state.tile_rect(from);
            let rect_b = state.tile_rect(to);
            let (w, h) = state.tile_size;
            let a = (rect_a.x(), rect_a.y());
            let b = (rect_b.x(), rect_b.y());
            let (x, y) = lerp(a, b, self.delta);
            let rect = Rect::new(x, y, w, h);
            render_unit(unit, rect, true, state, renderer);
        }
    }
}

fn lerp(a: (i32, i32), b: (i32, i32), delta: f32) -> (i32, i32) {
    let x = a.0 + ((b.0 - a.0) as f32 * delta) as i32;
    let y = a.1 + ((b.1 - a.1) as f32 * delta) as i32;
    (x, y)
}
