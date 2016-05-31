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
            selected: selected,
            options: labels,
            handler: handler,
        })
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
                let option = &self.options[self.selected];
                (self.handler)(Some(option.text()), state, queue);
            }
            Cancel => {
                (self.handler)(None, state, queue);
            }
            MoveCursorDown => {
                self.selected = (self.selected + 1) % self.options.len();
            }
            MoveCursorUp => {
                self.selected = (self.selected - 1) % self.options.len();
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State, renderer: &mut Renderer) {
        let (sx, sy) = self.pos;
        let line_spacing = self.options[0].font().recommended_line_spacing();
        let height = PAD * 2 + line_spacing as u32 * self.options.len() as u32;

        renderer.set_draw_color(Color::RGBA(200, 200, 255, 150));
        renderer.fill_rect(Rect::new(sx, sy, self.width, height)).unwrap();
        let mut y = sy + PAD as i32;
        let x = sx + PAD as i32;
        for (i, label) in self.options.iter_mut().enumerate() {
            if i == self.selected {
                renderer.set_draw_color(Color::RGB(255, 150, 0));
                let rect = Rect::new(x - PAD as i32 / 2, y, self.width - PAD, line_spacing as u32);
                renderer.fill_rect(rect).unwrap();
            }
            label.render(renderer, x, y);
            y += line_spacing;
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
