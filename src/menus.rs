use std::cmp;
use std::fmt::{self, Debug};
use std::rc::Rc;

use glorious::{Behavior, Label, Renderer};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::Font;

use common::{State, Message};

const PAD: u32 = 10;

pub struct ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    pos: (i32, i32),
    width: u32,
    line_spacing: u32,
    options: Vec<Label>,
    handler: F,
    selected: usize,
}

impl<F> ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    pub fn new<I>(options: I,
                  selected: usize,
                  pos: (i32, i32),
                  font: Rc<Font>,
                  state: &State,
                  handler: F)
                  -> Result<ModalMenu<F>, String>
        where I: IntoIterator<Item = String>
    {
        // TODO: Having to remember to scale ourselves is a bit annoying.
        let (_, scale_y) = state.resources.renderer().scale();
        let line_spacing = font.recommended_line_spacing();
        let line_spacing = (line_spacing as f32 / scale_y) as u32;

        let mut max_width = 0;
        let labels = options.into_iter()
            .map(|option| {
                let label = Label::new(font.clone(),
                                       option,
                                       (0, 0, 0, 0),
                                       state.resources.renderer());
                let (w, _) = label.size();
                max_width = cmp::max(max_width, w);
                label
            })
            .collect::<Vec<_>>();

        assert!(selected < labels.len(),
                "the selected option is out of bounds ({} of {})",
                selected,
                labels.len());

        Ok(ModalMenu {
            pos: pos,
            width: 2 * PAD + max_width,
            line_spacing: line_spacing,
            selected: selected,
            options: labels,
            handler: handler,
        })
    }

    fn confirm(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        let option = &self.options[self.selected];
        (self.handler)(Some(option.text()), state, queue);
    }

    fn cancel(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        (self.handler)(None, state, queue);
    }
}

impl<'a, F> Behavior<State<'a>> for ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            Confirm => {
                self.confirm(state, queue);
            }
            Cancel => {
                self.cancel(state, queue);
            }
            MoveCursorDown => {
                self.selected = (self.selected + 1) % self.options.len();
            }
            MoveCursorUp => {
                self.selected = (self.selected + self.options.len() - 1) % self.options.len();
            }
            MouseMovedTo(x, y) |
            LeftClickAt(x, y) => {
                let (outer_left, outer_top) = self.pos;

                let left = outer_left + PAD as i32;
                let top = outer_top + PAD as i32;

                let rx = x - left;
                let ry = y - top;

                let mut is_in_range = false;
                if 0 <= rx && rx <= ((self.width - PAD) as i32) && 0 <= ry {
                    let i = (ry / self.line_spacing as i32) as usize;
                    if i < self.options.len() {
                        is_in_range = true;
                        self.selected = i as usize;
                    }
                }

                if let LeftClickAt(..) = message {
                    if is_in_range {
                        self.confirm(state, queue);
                    } else {
                        self.cancel(state, queue);
                    }
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State, renderer: &mut Renderer) {
        let (sx, sy) = self.pos;
        let height = PAD * 2 + self.line_spacing * self.options.len() as u32;

        renderer.set_draw_color(Color::RGBA(200, 200, 255, 150));
        renderer.fill_rect(Rect::new(sx, sy, self.width, height)).unwrap();
        let mut y = sy + PAD as i32;
        let x = sx + PAD as i32;
        for (i, label) in self.options.iter_mut().enumerate() {
            if i == self.selected {
                renderer.set_draw_color(Color::RGB(255, 150, 0));
                let rect = Rect::new(x - PAD as i32 / 2, y, self.width - PAD, self.line_spacing);
                renderer.fill_rect(rect).unwrap();
            }
            label.render(renderer, x, y);
            y += self.line_spacing as i32;
        }
    }
}

impl<F> Debug for ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ModalMenu { .. }")
    }
}
