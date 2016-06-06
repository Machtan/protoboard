use std::cmp;
use std::fmt::{self, Debug};
use std::rc::Rc;

use glorious::{Behavior, Color, Label, Renderer};
use sdl2::rect::Rect;
use sdl2_ttf::Font;

use common::{Message, State};

const PAD: u32 = 10;
const COLOR_BG: Color = Color(0xcc, 0xcc, 0xff, 0x99);
const COLOR_TEXT: Color = Color(0x00, 0x00, 0x00, 0x00);
const COLOR_SELECTED: Color = Color(0xff, 0x99, 0x00, 0xff);

// TODO: Tune this for different platforms/hardware.
const SCROLL_TRESHOLD: i32 = 8;

pub struct ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    pos: (i32, i32),
    width: u32,
    line_spacing: u32,
    options: Vec<(Label, String)>,
    handler: F,
    selected: usize,
    confirm_areas: Vec<Rect>,
    amount_scrolled: i32,
}

impl<F> ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    pub fn new<I>(options: I,
                  selected: usize,
                  pos: (i32, i32),
                  font: Rc<Font>,
                  state: &State,
                  confirm_areas: Vec<Rect>,
                  handler: F)
                  -> Result<ModalMenu<F>, String>
        where I: IntoIterator<Item = String>
    {
        // TODO: Having to remember to scale ourselves is a bit annoying.
        let (_, scale_y) = state.resources.device().scale();
        let line_spacing = font.recommended_line_spacing();
        let line_spacing = (line_spacing as f32 / scale_y).round() as u32;

        let mut max_width = 0;
        let labels = options.into_iter()
            .map(|option| {
                let label = Label::new(&font, &option, COLOR_TEXT, state.resources.device());
                let (w, _) = label.size();
                max_width = cmp::max(max_width, w);
                (label, option)
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
            confirm_areas: confirm_areas,
            amount_scrolled: 0,
        })
    }

    fn handle(&mut self, selected: Option<usize>, state: &mut State, queue: &mut Vec<Message>) {
        let options = &self.options;
        let option = selected.map(|i| &options[i].1[..]);
        (self.handler)(option, state, queue);
    }

    fn confirm(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        let i = self.selected;
        self.handle(Some(i), state, queue);
    }

    fn cancel(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        self.handle(None, state, queue);
    }

    fn render_options(&self, renderer: &mut Renderer) {
        // This is just here to demonstrate, that mutable access to self
        // is not needed.

        let (sx, sy) = self.pos;
        let height = PAD * 2 + self.line_spacing * self.options.len() as u32;

        renderer.set_draw_color(COLOR_BG);
        renderer.fill_rect(Rect::new(sx, sy, self.width, height)).unwrap();
        let mut y = sy + PAD as i32;
        let x = sx + PAD as i32;
        for (i, &(ref label, _)) in self.options.iter().enumerate() {
            if i == self.selected {
                renderer.set_draw_color(COLOR_SELECTED);
                let rect = Rect::new(x - PAD as i32 / 2, y, self.width - PAD, self.line_spacing);
                renderer.fill_rect(rect).unwrap();
            }
            label.render(renderer, x, y);
            y += self.line_spacing as i32;
        }
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
            Cancel |
            RightClickAt(_, _) => {
                self.cancel(state, queue);
            }
            MoveCursorDown => {
                self.selected = (self.selected + 1) % self.options.len();
            }
            MoveCursorUp => {
                self.selected = (self.selected + self.options.len() - 1) % self.options.len();
            }
            MouseScroll(_relx, rely) => {
                // Reset on new direction
                if rely > 0 && self.amount_scrolled < 0 || rely < 0 && self.amount_scrolled > 0 {
                    self.amount_scrolled = 0;
                }
                self.amount_scrolled += rely;
                if self.amount_scrolled >= SCROLL_TRESHOLD {
                    self.selected = cmp::min(self.selected + 1, self.options.len() - 1);
                    self.amount_scrolled = 0;
                } else if self.amount_scrolled <= -SCROLL_TRESHOLD {
                    self.selected = self.selected.saturating_sub(1);
                    self.amount_scrolled = 0;
                }
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

                if let LeftClickAt(x, y) = message {
                    if is_in_range {
                        self.confirm(state, queue);
                    } else {
                        let valid = self.confirm_areas.iter().any(|a| a.contains((x, y)));
                        if valid {
                            self.confirm(state, queue);
                        } else {
                            self.cancel(state, queue);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State, renderer: &mut Renderer) {
        self.render_options(renderer);
    }
}

impl<F> Debug for ModalMenu<F>
    where F: FnMut(Option<&str>, &mut State, &mut Vec<Message>)
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ModalMenu { .. }")
    }
}
