use glorious::Behavior;
use common::{State, Message};
use sdl2::rect::Rect;
use sdl2::render::Renderer;

pub struct Cursor {
    col: u32,
    row: u32,
    grid_rows: u32,
    grid_cols: u32,
    size: (u32, u32),
}

impl Cursor {
    pub fn new(col: u32, row: u32, grid_rows: u32, grid_cols: u32, size: (u32, u32)) -> Self {
        Cursor {
            col: col,
            row: row,
            grid_rows: grid_rows,
            grid_cols: grid_cols,
            size: size,
        }
    }
}

impl Behavior for Cursor {
    type State = State;
    type Message = Message;
    
    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, _state: &mut Self::State, _new_messages: &mut Vec<Self::Message>) {
        // Do nothing by default
    }

    /// Updates the object each frame.
    fn update(&mut self, _state: &mut Self::State, _queue: &mut Vec<Self::Message>) {
        // Do nothing by default
    }

    /// Handles new messages since the last frame.
    fn handle(&mut self,
              state: &mut Self::State,
              messages: &[Self::Message],
              new_messages: &mut Vec<Self::Message>) {
        use common::Message::*;
        // Do nothing by default
        for message in messages {
            match *message {
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
                    new_messages.push(new_message);
                }
                Cancel => {
                    let new_message = CursorCancel(self.col, self.row);
                    new_messages.push(new_message);
                }
                _ => {}
            }
        }
    }

    /// Renders the object.
    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        let x = (self.col * self.size.0) as i32;
        let grid_height = self.grid_rows * self.size.1;
        let y = (grid_height - self.size.1 - (self.row * self.size.1)) as i32;
        let sprite = state.resources.sprite("marker").unwrap();
        sprite.render(renderer, x, y, Some(self.size));
    }
}