use glorious::{Behavior, Renderer, Sprite};

use common::{MARKER_PATH, State, Message};

#[derive(Debug)]
pub struct Cursor {
    col: u32,
    row: u32,
    grid_rows: u32,
    grid_cols: u32,
    size: (u32, u32),
    hidden: bool,
}

impl Cursor {
    pub fn new(col: u32, row: u32, grid_rows: u32, grid_cols: u32, size: (u32, u32)) -> Cursor {
        Cursor {
            col: col,
            row: row,
            grid_rows: grid_rows,
            grid_cols: grid_cols,
            size: size,
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
                if self.row < (self.grid_rows - 1) {
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
                if self.col < (self.grid_cols - 1) {
                    self.col += 1;
                }
            }
            MoveCursorTo(col, row) => {
                self.col = col;
                self.row = row;
            }
            Confirm => {
                let new_message = CursorConfirm(self.col, self.row);
                queue.push(new_message);
            }
            Cancel => {
                let new_message = CursorCancel(self.col, self.row);
                queue.push(new_message);
            }
            ShowCursor => {
                self.hidden = false;
            }
            HideCursor => {
                self.hidden = true;
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        if self.hidden {
            return;
        }
        let x = (self.col * self.size.0) as i32;
        let grid_height = self.grid_rows * self.size.1;
        let y = (grid_height - self.size.1 - (self.row * self.size.1)) as i32;
        let sprite = Sprite::new(state.resources.texture(MARKER_PATH), None);
        sprite.render(renderer, x, y, Some(self.size));
    }
}
