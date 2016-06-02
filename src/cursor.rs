use glorious::{Behavior, Renderer, Sprite};

use resources::MARKER_PATH;
use common::{State, Message};

#[derive(Debug)]
pub struct Cursor {
    col: u32,
    row: u32,
    grid_size: (u32, u32),
    tile_size: (u32, u32),
    hidden: bool,
}

impl Cursor {
    pub fn new(tile: (u32, u32), grid_size: (u32, u32), tile_size: (u32, u32)) -> Cursor {
        let (col, row) = tile;
        Cursor {
            col: col,
            row: row,
            grid_size: grid_size,
            tile_size: tile_size,
            hidden: false,
        }
    }
}

impl<'a> Behavior<State<'a>> for Cursor {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, _state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            MoveCursorUp => {
                if self.row < (self.grid_size.1 - 1) {
                    self.row += 1;
                }
            }
            MoveCursorDown => {
                if self.row > 0 {
                    self.row -= 1;
                }
            }
            MoveCursorLeft => {
                if self.col > 0 {
                    self.col -= 1;
                }
            }
            MoveCursorRight => {
                if self.col < (self.grid_size.0 - 1) {
                    self.col += 1;
                }
            }
            MoveCursorTo((col, row)) => {
                self.col = col;
                self.row = row;
            }
            Confirm => {
                let new_message = CursorConfirm((self.col, self.row));
                queue.push(new_message);
            }
            Cancel => {
                let new_message = CursorCancel((self.col, self.row));
                queue.push(new_message);
            }
            ShowCursor => {
                self.hidden = false;
            }
            HideCursor => {
                self.hidden = true;
            }
            MouseMovedTo(x, y) => {
                assert!(x >= 0 && y >= 0);
                let (w, h) = self.tile_size;
                let (_, rows) = self.grid_size;
                self.col = (x as u32 - (x as u32 % w)) / w;
                self.row = rows - 1 - (y as u32 - (y as u32 % h)) / h;
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        if self.hidden {
            return;
        }
        let (w, h) = self.tile_size;
        let (_, gh) = self.grid_size;
        let x = (self.col * w) as i32;
        let grid_height = gh * h;
        let y = (grid_height - h - (self.row * h)) as i32;
        let sprite = Sprite::new(state.resources.texture(MARKER_PATH), None);
        sprite.render(renderer, x, y, Some(self.tile_size));
    }
}
