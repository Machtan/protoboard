use std::fmt::{self, Debug};

use glorious::Behavior;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use common::{State, Message};

const PAD: u32 = 10;

#[derive(Clone)]
pub struct ModalMenu<F>
    where F: FnMut(&str, &mut Vec<Message>)
{
    pos: (i32, i32),
    width: u32,
    font: String,
    options: Vec<String>,
    label_ids: Vec<String>,
    handler: F,
    selected: usize,
}

impl<F> ModalMenu<F>
    where F: FnMut(&str, &mut Vec<Message>)
{
    pub fn new(options: &[&str],
               selected: usize,
               pos: (i32, i32),
               font: &str,
               handler: F)
               -> Result<ModalMenu<F>, String> {
        assert!(!options.is_empty(),
                "a modal menu must have at least one option");
        assert!(selected < options.len(),
                "the selected option is out of bounds ({} of {})",
                selected,
                options.len());
        let font = font.to_owned();
        let label_ids = options.iter()
            .map(|option| {
                let mut id = font.clone();
                id.push('/');
                id.push_str(option);
                id
            })
            .collect();

        Ok(ModalMenu {
            pos: pos,
            width: 0,
            font: font,
            selected: selected,
            options: options.iter().map(|&s| s.to_owned()).collect(),
            label_ids: label_ids,
            handler: handler,
        })
    }
}

impl<F> Behavior for ModalMenu<F>
    where F: FnMut(&str, &mut Vec<Message>)
{
    type State = State;
    type Message = Message;

    /// Initializes the object when it is added to the game.
    fn initialize(&mut self,
                  state: &mut State,
                  _queue: &mut Vec<Message>,
                  renderer: &mut Renderer) {
        let mut max_width = 0;
        for (i, id) in self.label_ids.iter().enumerate() {
            state.resources
                .create_label(id, &self.font, &self.options[i], (0, 0, 0, 0), renderer)
                .expect("could not create label");
            let label = state.resources.label(id).unwrap();
            let w = label.rect.width();
            if w > max_width {
                max_width = w;
            }
        }
        self.width = max_width + 2 * PAD;
    }

    /// Updates the object each frame.
    fn update(&mut self, _state: &mut State, _queue: &mut Vec<Message>) {
        // TODO
    }

    /// Handles new messages since the last frame.
    fn handle(&mut self,
              state: &mut State,
              message: Message,
              queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            Confirm => {
                let option = &self.options[self.selected];
                (self.handler)(option, queue);
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&self, state: &State, renderer: &mut Renderer) {
        let (sx, sy) = self.pos;
        let font = state.resources.font(&self.font).unwrap();
        let height = PAD * 2 + font.recommended_line_spacing() as u32 * self.options.len() as u32;

        renderer.set_draw_color(Color::RGBA(200, 200, 255, 150));
        renderer.fill_rect(Rect::new(sx, sy, self.width, height)).unwrap();
        let mut y = sy + PAD as i32;
        let x = sx + PAD as i32;
        for (i, id) in self.label_ids.iter().enumerate() {
            let label = state.resources.label(id).unwrap();
            if i == self.selected {
                renderer.set_draw_color(Color::RGB(255, 150, 0));
                renderer.fill_rect(label.rect).unwrap();
            }
            label.render(renderer, x, y, None);
            y += font.recommended_line_spacing();
        }
    }
}

impl<F> Debug for ModalMenu<F>
    where F: FnMut(&str, &mut Vec<Message>)
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ModalMenu { .. }")
    }
}
