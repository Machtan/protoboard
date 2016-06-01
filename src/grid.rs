use std::fmt::{self, Debug};

use glorious::{Behavior, Renderer, Sprite};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use common::{FIRA_SANS_PATH, State, Message};
use unit::Unit;
use menus::ModalMenu;
use target_selector::TargetSelector;

#[derive(Debug, Clone)]
pub struct Tile {
    unit: Option<Unit>,
    terrain: Option<()>,
}

impl Tile {
    #[inline]
    pub fn new(unit: Option<Unit>) -> Tile {
        Tile {
            unit: unit,
            terrain: None,
        }
    }
}

pub struct Grid {
    size: (u32, u32),
    cell_size: (u32, u32),
    contents: Box<[Tile]>,
    selected_unit: Option<(u32, u32)>,
}

impl Grid {
    pub fn new(size: (u32, u32), cell_size: (u32, u32)) -> Grid {
        let contents = vec![Tile::new(None); size.0 as usize * size.1 as usize];
        Grid {
            size: size,
            cell_size: cell_size,
            contents: contents.into_boxed_slice(),
            selected_unit: None,
        }
    }

    fn index(&self, cell: (u32, u32)) -> usize {
        let (col, row) = cell;
        let (cols, rows) = self.size;
        assert!(row < rows && col < cols);
        col as usize * rows as usize + row as usize
    }

    pub fn tile(&self, cell: (u32, u32)) -> &Tile {
        &self.contents[self.index(cell)]
    }

    pub fn tile_mut(&mut self, cell: (u32, u32)) -> &mut Tile {
        &mut self.contents[self.index(cell)]
    }

    pub fn unit(&self, cell: (u32, u32)) -> Option<&Unit> {
        self.tile(cell).unit.as_ref()
    }

    pub fn unit_mut(&mut self, cell: (u32, u32)) -> Option<&mut Unit> {
        self.tile_mut(cell).unit.as_mut()
    }

    /// Adds a unit to the grid.
    pub fn add_unit(&mut self, unit: Unit, cell: (u32, u32)) {
        let tile = self.tile_mut(cell);
        assert!(tile.unit.is_none());
        tile.unit = Some(unit);
    }

    /// Finds tiles attackable by the given unit if moved to the given position.
    fn find_attackable(&self, unit: &Unit, cell: (u32, u32)) -> Vec<((u32, u32), Tile)> {
        let mut attackable = Vec::new();
        for target_cell in unit.attack.cells_in_range(cell, self.size) {
            if self.unit(target_cell).is_some() {
                attackable.push((target_cell, self.tile(cell).clone()));
            }
        }
        attackable
    }

    /// Moves a unit from the source to the destination.
    fn move_unit(&mut self, from: (u32, u32), to: (u32, u32)) {
        if from == to {
            return;
        }
        assert!(self.unit(to).is_none(), "Transport units not supported!");
        assert!(self.unit(from).is_some(), "No unit at move origin");
        let i = self.index(from);
        let j = self.index(to);
        self.contents.swap(i, j);
    }

    /// Opens the target selection modal for the unit at Cell.
    /// The origin is used to return to the menu when cancelling.
    fn select_target(&mut self,
                     origin: (u32, u32),
                     cell: (u32, u32),
                     state: &mut State,
                     queue: &mut Vec<Message>) {
        info!("Selecting target...");
        let unit = self.unit(cell).unwrap().clone();
        let targets = self.find_attackable(&unit, cell);
        let selector = TargetSelector::new(unit, cell, origin, self.size, self.cell_size, targets);
        queue.push(Message::HideCursor);
        state.push_modal(Box::new(selector), queue);
    }

    /// Moves the selected unit from origin to target and opens up the action menu.
    /// If the menu is cancelled, the unit moves back.
    fn move_unit_and_act(&mut self,
                         origin: (u32, u32),
                         target: (u32, u32),
                         state: &mut State,
                         queue: &mut Vec<Message>) {
        use common::Message::*;
        assert!(self.selected_unit == Some(origin),
                "The moved unit isn't selected");
        if self.unit(target).is_none() || target == origin {
            self.move_unit(origin, target);
            debug!("Moved unit from {:?} to {:?}", origin, target);

            let targets = self.find_attackable(self.unit(target).unwrap(), target);
            let mut options = Vec::new();
            if !targets.is_empty() {
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
                        state.pop_modal(queue);
                        queue.push(MoveCursorTo(target));
                        queue.push(SelectTarget(origin, target));
                    }
                    Some("Wait") => {
                        info!("Wait!");
                        state.pop_modal(queue);
                        queue.push(UnitSpent(target));
                        queue.push(Deselect);
                        queue.push(MoveCursorTo(target));
                        queue.push(ShowCursor);
                    }
                    None => {
                        info!("Cancel!");
                        state.pop_modal(queue);
                        queue.push(MoveUnit(target, origin));
                        queue.push(MoveCursorTo(origin));
                        queue.push(SelectUnit(origin));
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

    /// Handles a confirm press at the given target cell when a unit is selected.
    fn on_confirm(&mut self, target: (u32, u32), state: &mut State, queue: &mut Vec<Message>) {
        if let Some(origin) = self.selected_unit {
            self.move_unit_and_act(origin, target, state, queue);
        } else if self.unit(target).is_some() {
            self.selected_unit = Some(target);
        }
    }
}

impl<'a> Behavior<State<'a>> for Grid {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            CursorConfirm(cell) => {
                self.on_confirm(cell, state, queue);
            }
            LeftClickAt(x, y) => {
                assert!(x >= 0 && y >= 0);
                let (w, h) = self.cell_size;
                let (_, rows) = self.size;
                let col = (x as u32 - (x as u32 % w)) / w;
                let row = rows - 1 - (y as u32 - (y as u32 % h)) / h;
                queue.push(MoveCursorTo((col, row)));
                self.on_confirm((col, row), state, queue);
            }
            CursorCancel(..) => {
                if self.selected_unit.is_some() {
                    self.selected_unit = None;
                }
            }
            UnitSpent(cell) => {
                self.unit_mut(cell)
                    .expect("No unit at the spent cell!")
                    .spent = true;
            }
            MoveUnit(from, to) => {
                self.move_unit(from, to);
            }
            SelectUnit(cell) => {
                assert!(self.unit(cell).is_some(),
                        "The tile for the selected unit is empty!");
                self.selected_unit = Some(cell);
            }
            Deselect => {
                assert!(self.selected_unit.is_some(),
                        "Received deselect with no unit selected");
                self.selected_unit = None;
            }
            SelectTarget(origin, cell) => {
                self.select_target(origin, cell, state, queue);
            }
            MoveUnitAndAct(origin, destination) => {
                self.move_unit_and_act(origin, destination, state, queue);
            }
            DestroyUnit(origin) => {
                let tile = self.tile_mut(origin);
                info!("Unit at {:?} destroyed! {:?}",
                      origin,
                      tile.unit.as_ref().expect("no unit to destroy"));
                tile.unit = None;
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State<'a>, renderer: &mut Renderer) {
        let (cols, rows) = self.size;
        let (cw, ch) = self.cell_size;
        let grid_height = rows * ch;
        for col in 0..cols {
            for row in 0..rows {
                let x = col * cw;
                let y = grid_height - ch - (row * ch);
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

                let rect = Rect::new(x as i32, y as i32, cw, ch);
                // TODO: When can `fill_rect` fail?
                renderer.fill_rect(rect).unwrap();

                let tile = self.tile((col, row));
                if let Some(()) = tile.terrain {
                }
                if let Some(ref obj) = tile.unit {
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
