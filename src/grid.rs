use glorious::Behavior;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use common::{State, Message};
use unit::Unit;
use menus::ModalMenu;

#[derive(Clone, Debug)]
pub struct GridField {
    unit: Option<Unit>,
    terrain: Option<()>,
}

impl GridField {
    #[inline]
    pub fn new(unit: Option<Unit>) -> GridField {
        GridField {
            unit: unit,
            terrain: None,
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    cols: u32,
    rows: u32,
    cell_size: (u32, u32),
    contents: Box<[GridField]>,
    selected_unit: Option<(u32, u32)>,
}

impl Grid {
    pub fn new(cols: u32, rows: u32, cell_size: (u32, u32)) -> Grid {
        let contents = vec![GridField::new(None); cols as usize * rows as usize];
        Grid {
            cols: cols,
            rows: rows,
            cell_size: cell_size,
            contents: contents.into_boxed_slice(),
            selected_unit: None,
        }
    }

    fn index(&self, col: u32, row: u32) -> usize {
        assert!(row < self.rows && col < self.cols);
        col as usize * self.rows as usize + row as usize
    }

    pub fn field(&self, col: u32, row: u32) -> &GridField {
        &self.contents[self.index(col, row)]
    }

    pub fn field_mut(&mut self, col: u32, row: u32) -> &mut GridField {
        &mut self.contents[self.index(col, row)]
    }

    pub fn unit(&self, col: u32, row: u32) -> Option<&Unit> {
        self.field(col, row).unit.as_ref()
    }

    pub fn unit_mut(&mut self, col: u32, row: u32) -> Option<&mut Unit> {
        self.field_mut(col, row).unit.as_mut()
    }

    pub fn add_unit(&mut self, unit: Unit, col: u32, row: u32) {
        let field = self.field_mut(col, row);
        assert!(field.unit.is_none());
        field.unit = Some(unit);
    }
    
    fn find_attackable(&self, unit: &Unit, col: u32, row: u32) -> Vec<(u32, u32)> {
        let mut attackable = Vec::new();
        for (tc, tr) in unit.attack.cells_in_range(col, row, (self.cols, self.rows)) {
            if self.unit(tc, tr).is_some() {
                attackable.push((tc, tr));
            }
        }
        attackable
    }

    fn move_unit_to(&mut self, col: u32, row: u32, state: &mut State, _queue: &mut Vec<Message>) {
        use common::Message::*;
        let (ucol, urow) = self.selected_unit.expect("no unit was selected");
        if col == ucol && row == urow {
            self.selected_unit = None;
        } else if self.unit(col, row).is_none() {
            let i = self.index(ucol, urow);
            let j = self.index(col, row);

            assert!(self.contents[i].unit.is_some(),
                    "selected unit points to vacant tile");
            self.contents.swap(i, j);
            self.selected_unit = None;

            debug!("Moved unit from ({}, {}) to ({}, {})", ucol, urow, col, row);

            let menu = ModalMenu::new(&["Attack", "Wait"],
                                      0,
                                      (50, 50),
                                      "firasans",
                                      state,
                                      |option, state, _queue| {
                match option {
                    "Attack" => {
                        info!("Attack!");
                        state.pop_modal();
                    }
                    "Wait" => {
                        info!("Wait!");
                        state.pop_modal();
                    }
                    _ => unreachable!(),
                }
            })
                .expect("could not create menu");
            state.push_modal(Box::new(menu));
        }
    }

    fn on_confirm(&mut self, col: u32, row: u32, state: &mut State, queue: &mut Vec<Message>) {
        if self.selected_unit.is_some() {
            self.move_unit_to(col, row, state, queue);
        } else if self.unit(col, row).is_some() {
            self.selected_unit = Some((col, row));
        }
    }
}

impl Behavior for Grid {
    type State = State;
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            CursorConfirm(col, row) => {
                self.on_confirm(col, row, state, queue);
            }
            LeftClickAt(x, y) => {
                assert!(x >= 0 && y >= 0);
                let (w, h) = self.cell_size;
                let col = (x as u32 - (x as u32 % w)) / w;
                let row = self.rows - 1 - (y as u32 - (y as u32 % h)) / h;
                queue.push(MoveCursorTo(col, row));
                self.on_confirm(col, row, state, queue);
            }
            CursorCancel(..) => {
                if self.selected_unit.is_some() {
                    self.selected_unit = None;
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State, renderer: &mut Renderer) {
        let grid_height = self.rows * self.cell_size.1;
        for col in 0..self.cols {
            for row in 0..self.rows {
                let x = col * self.cell_size.0;
                let y = grid_height - self.cell_size.1 - (row * self.cell_size.1);
                // sprite.render(renderer, x as i32, y as i32, Some(self.cell_size));
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

                let rect = Rect::new(x as i32, y as i32, self.cell_size.0, self.cell_size.1);
                // TODO: When can `fill_rect` fail?
                renderer.fill_rect(rect).unwrap();

                let field = self.field(col, row);
                if let Some(()) = field.terrain {
                }
                if let Some(ref obj) = field.unit {
                    let sprite = state.resources.sprite(obj.texture).unwrap();
                    sprite.render(renderer, x as i32, y as i32, Some(self.cell_size));
                }
            }
        }
    }
}
