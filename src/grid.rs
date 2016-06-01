use std::fmt::{self, Debug};

use glorious::{Behavior, Renderer, Sprite};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use common::{FIRA_SANS_PATH, State, Message};
use unit::Unit;
use menus::ModalMenu;
use target_selector::TargetSelector;

#[derive(Debug, Clone)]
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

    fn find_attackable(&self, unit: &Unit, col: u32, row: u32) 
            -> Vec<((u32, u32), GridField)> {
        let mut attackable = Vec::new();
        for (tc, tr) in unit.attack.cells_in_range(col, row, (self.cols, self.rows)) {
            if self.unit(tc, tr).is_some() {
                attackable.push(((tc, tr), self.field(col, row).clone()));
            }
        }
        attackable
    }
    
    fn move_unit(&mut self, from: (u32, u32), to: (u32, u32)) {
        if from == to {
            return;
        }
        let (src_col, src_row) = from;
        let (dst_col, dst_row) = to;
        assert!(self.unit(dst_col, dst_row).is_none(),
                "Transport units not supported!");
        assert!(self.unit(src_col, src_row).is_some(),
                "No unit at move origin");
        let i = self.index(src_col, src_row);
        let j = self.index(dst_col, dst_row);
        self.contents.swap(i, j);
    }
    
    fn select_target(&mut self, cell: (u32, u32), state: &mut State, queue: &mut Vec<Message>) {
        info!("Selecting target...");
        let (col, row) = cell;
        let unit = self.unit(col, row).unwrap().clone();
        let targets = self.find_attackable(&unit, col, row);
        let selector = TargetSelector::new(
            unit,
            (col, row),
            (self.cols, self.rows),
            self.cell_size, 
            targets);
        queue.push(Message::Deselect);
        queue.push(Message::HideCursor);
        state.push_modal(Box::new(selector), queue);
    }

    fn do_action_at(&mut self, col: u32, row: u32, state: &mut State, queue: &mut Vec<Message>) {
        use common::Message::*;
        let (ucol, urow) = self.selected_unit.expect("no unit was selected");
        if self.unit(col, row).is_none() || (col == ucol && row == urow) {
            self.move_unit((ucol, urow), (col, row));
            debug!("Moved unit from ({}, {}) to ({}, {})", ucol, urow, col, row);
            
            let targets = self.find_attackable(self.unit(col, row).unwrap(),
                col, row);
            let mut options = Vec::new();
            if ! targets.is_empty() {
                options.push("Attack");
            }
            options.push("Wait");
            let menu = ModalMenu::new(options.iter().map(|&s| s.to_owned()),
                                      0,
                                      (50, 50),
                                      state.resources.font(FIRA_SANS_PATH, 16),
                                      state,
                                      move |option, state, queue| {
                match option {
                    Some("Attack") => {
                        info!("Attack!");
                        // TODO: Just to prevent a crash after failing to attack.
                        state.pop_modal(queue);
                        queue.push(MoveCursorTo(col, row));
                        queue.push(SelectTarget(col, row));
                    }
                    Some("Wait") => {
                        info!("Wait!");
                        state.pop_modal(queue);
                        queue.push(UnitSpent(col, row));
                        queue.push(Deselect);
                        queue.push(MoveCursorTo(col, row));
                        queue.push(ShowCursor);
                    }
                    None => {
                        info!("Cancel!");
                        state.pop_modal(queue);
                        queue.push(MoveUnit((col, row), (ucol, urow)));
                        queue.push(MoveCursorTo(ucol, urow));
                        queue.push(SelectUnit(ucol, urow));
                        queue.push(ShowCursor);
                    }
                    _ => unreachable!(),
                }
            })
                .expect("could not create menu");
            queue.push(HideCursor);
            state.push_modal(Box::new(menu), queue);
        }
    }

    fn on_confirm(&mut self, col: u32, row: u32, state: &mut State, queue: &mut Vec<Message>) {
        if self.selected_unit.is_some() {
            self.do_action_at(col, row, state, queue);
        } else if self.unit(col, row).is_some() {
            self.selected_unit = Some((col, row));
        }
    }
}

impl<'a> Behavior<State<'a>> for Grid {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
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
            UnitSpent(col, row) => {
                self.unit_mut(col, row)
                    .expect("No unit at the spent cell!")
                    .spent = true;
            }
            MoveUnit(from, to) => {
                self.move_unit(from, to);
            }
            SelectUnit(col, row) => {
                assert!(self.unit(col, row).is_some(),
                        "The field for the selected unit is empty!");
                self.selected_unit = Some((col, row));
            }
            Deselect => {
                assert!(self.selected_unit.is_some(),
                        "Received deselect with no unit selected");
                self.selected_unit = None;
            }
            SelectTarget(col, row) => {
                self.select_target((col, row), state, queue);
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State<'a>, renderer: &mut Renderer) {
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
                    if obj.spent {
                        renderer.set_draw_color(Color::RGB(100, 150, 100));
                        renderer.fill_rect(rect).unwrap();
                    }
                    let sprite = Sprite::new(obj.texture.clone(), None);
                    sprite.render(renderer, x as i32, y as i32, Some(self.cell_size));
                }
            }
        }
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Grid {{ .. }}")
    }
}
