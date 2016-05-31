use std::fmt::{self, Debug};

use glorious::{Behavior, Label};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

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
    pub fn new(options: &[&str],
               selected: usize,
               pos: (i32, i32),
               font: &str,
               state: &State,
               handler: F)
               -> Result<ModalMenu<F>, String> {
        assert!(!options.is_empty(),
                "a modal menu must have at least one option");
        assert!(selected < options.len(),
                "the selected option is out of bounds ({} of {})",
                selected,
                options.len());

        let mut labels = Vec::new();
        let mut max_width = 0;
        let loaded_font = state.resources.font(font).expect("Modal font not found");
        for option in options {
            let label = Label::new(font, loaded_font, option, (0, 0, 0, 0));
            let (w, _) = label.size();
            if w > max_width {
                max_width = w;
            }
            labels.push(label);
        }
        Ok(ModalMenu {
            pos: pos,
            width: 2 * PAD + max_width,
            selected: selected,
            options: labels,
            handler: handler,
        })
    }
}

impl<F> Behavior for ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    type State = State;
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
    fn render(&mut self, state: &State, renderer: &mut Renderer) {
        let (sx, sy) = self.pos;
        let line_spacing = {
            let font_name = self.options[0].font();
            let font = state.resources.font(font_name).unwrap();
            font.recommended_line_spacing()
        };
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
            label.render(renderer, x, y, &state.resources);
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
