use glorious::Behavior;
use sdl2::render::Renderer;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use common::{GameObject, State, Message};
use unit::Unit;

pub struct GridField {
    unit: Option<Unit>,
    terrain: Option<()>,
}

impl GridField {
    pub fn new(unit: Option<Unit>) -> Self {
        GridField {
            unit: unit,
            terrain: None,
        }
    }
}

pub struct Grid {
    cols: u32,
    rows: u32,
    cell_size: (u32, u32),
    contents: Vec<Vec<GridField>>,
    selected_unit: Option<(u32, u32)>,
}

impl Grid {
    pub fn new(cols: u32, rows: u32, cell_size: (u32, u32)) -> Self {
        let mut contents = Vec::new();
        for x in 0..cols {
            let mut col = Vec::new();
            for y in 0..rows {
                col.push(GridField::new(None));
            }
            contents.push(col);
        }
        Grid {
            cols: cols,
            rows: rows,
            cell_size: cell_size,
            contents: contents,
            selected_unit: None,
        }
    }
    
    pub fn field(&mut self, col: u32, row: u32) -> &mut GridField {
        &mut self.contents[col as usize][row as usize]
    }
    
    pub fn unit(&mut self, col: u32, row: u32) -> Option<&mut Unit> {
        if let Some(ref mut unit) = self.contents[col as usize][row as usize].unit {
            Some(unit)
        } else {
            None
        }
    }
    
    pub fn add_unit(&mut self, unit: Unit, col: u32, row: u32) -> Result<(), String> {
        if col > (self.cols - 1) {
            return Err(format!("Column {} > {}", col, self.cols-1));
        }
        if row > (self.rows - 1) {
            return Err(format!("Row {} > {}", row, self.rows - 1));
        }
        self.contents[col as usize][row as usize].unit = Some(unit);
        Ok(())
    }
    
    fn move_unit_to(&mut self, col: u32, row: u32) {
        let (ucol, urow) = self.selected_unit.unwrap();
        let occupied = self.unit(col, row).is_some();
        if col == ucol && row == urow {
            self.selected_unit = None;
        }
        if ! occupied {
            self.field(col, row).unit = Some(self.unit(ucol, urow).unwrap().clone());
            self.selected_unit = None;
            self.field(ucol, urow).unit = None;
            println!("Moved unit from ({}, {}) to ({}, {})", ucol, urow, col, row);
        }
    }
    
    fn on_confirm(&mut self, col: u32, row: u32) {
        if self.selected_unit.is_some() {
            self.move_unit_to(col, row);
        } else if self.unit(col, row).is_some() {
            self.selected_unit = Some((col, row));
        }
    }
}

impl Behavior for Grid {
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
                CursorConfirm(col, row) => {
                    //println!("Confirm @ ({}, {})", col, row);
                    self.on_confirm(col, row);
                }
                LeftClickAt(x, y) => {
                    let (w, h) = self.cell_size;
                    let col = (x - (x % w)) / w;
                    let row = (y - (y % h)) / h;
                    self.on_confirm(col, row);
                }
                CursorCancel(col, row) => {
                    //println!("Cancel @ ({}, {})", col, row);
                    if self.selected_unit.is_some() {
                        self.selected_unit = None;
                    }
                }
                _ => {}
            }
        }
    }

    /// Renders the object.
    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        let grid_height = self.rows * self.cell_size.1;
        for col in 0..self.cols {
            for row in 0..self.rows {
                let x = col * self.cell_size.0;
                let y = grid_height - self.cell_size.1 - (row * self.cell_size.1);
                //sprite.render(renderer, x as i32, y as i32, Some(self.cell_size));
                if (col + row) % 2 == 0 {
                    renderer.set_draw_color(Color::RGB(210, 210, 210));
                } else {
                    renderer.set_draw_color(Color::RGB(255, 255, 255));
                }
                if let Some((ucol, urow)) = self.selected_unit {
                    if ucol == col && urow == row {
                        renderer.set_draw_color(Color::RGB(0, 255, 0));
                    }
                }
                renderer.fill_rect(Rect::new(x as i32, y as i32, 
                    self.cell_size.0, self.cell_size.1));
                let ref field = self.contents[col as usize][row as usize];
                if let Some(()) = field.terrain {
                    
                }
                if let Some(ref obj) = field.unit {
                    let sprite = state.resources.sprite(obj.texture).unwrap();
                    sprite.render(renderer, x as i32, y as i32, Some(self.cell_size));
                }
            }
        }
        
        let label = state.resources.label("hello_world").unwrap();
        label.render(renderer, 200, 200, None);
    }
}