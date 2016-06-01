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
    tile_size: (u32, u32),
    tiles: Box<[Tile]>,
    selected_unit: Option<(u32, u32)>,
}

impl Grid {
    pub fn new(size: (u32, u32), tile_size: (u32, u32)) -> Grid {
        let tiles = vec![Tile::new(None); size.0 as usize * size.1 as usize];
        Grid {
            size: size,
            tile_size: tile_size,
            tiles: tiles.into_boxed_slice(),
            selected_unit: None,
        }
    }

    fn index(&self, pos: (u32, u32)) -> usize {
        let (col, row) = pos;
        let (cols, rows) = self.size;
        assert!(row < rows && col < cols);
        col as usize * rows as usize + row as usize
    }

    pub fn tile(&self, pos: (u32, u32)) -> &Tile {
        &self.tiles[self.index(pos)]
    }

    pub fn tile_mut(&mut self, pos: (u32, u32)) -> &mut Tile {
        &mut self.tiles[self.index(pos)]
    }

    pub fn unit(&self, pos: (u32, u32)) -> Option<&Unit> {
        self.tile(pos).unit.as_ref()
    }

    pub fn unit_mut(&mut self, pos: (u32, u32)) -> Option<&mut Unit> {
        self.tile_mut(pos).unit.as_mut()
    }

    /// Adds a unit to the grid.
    pub fn add_unit(&mut self, unit: Unit, pos: (u32, u32)) {
        let tile = self.tile_mut(pos);
        assert!(tile.unit.is_none());
        tile.unit = Some(unit);
    }

    /// Finds tiles attackable by the given unit if moved to the given position.
    fn find_attackable(&self, unit: &Unit, pos: (u32, u32)) -> Vec<((u32, u32), Tile)> {
        let mut attackable = Vec::new();
        for target_pos in unit.attack.tiles_in_range(pos, self.size) {
            if self.unit(target_pos).is_some() {
                attackable.push((target_pos, self.tile(pos).clone()));
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
        self.tiles.swap(i, j);
    }

    /// Opens the target selection modal for the unit at Cell.
    /// The origin is used to return to the menu when cancelling.
    fn select_target(&mut self,
                     origin: (u32, u32),
                     pos: (u32, u32),
                     state: &mut State,
                     queue: &mut Vec<Message>) {
        info!("Selecting target...");
        let unit = self.unit(pos).unwrap().clone();
        let targets = self.find_attackable(&unit, pos);
        let selector = TargetSelector::new(unit, pos, origin, self.size, self.tile_size, targets);
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

    /// Handles a confirm press at the given target tile when a unit is selected.
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
            CursorConfirm(pos) => {
                self.on_confirm(pos, state, queue);
            }
            LeftClickAt(x, y) => {
                assert!(x >= 0 && y >= 0);
                let (w, h) = self.tile_size;
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
            UnitSpent(pos) => {
                self.unit_mut(pos)
                    .expect("No unit on the spent tile!")
                    .spent = true;
            }
            MoveUnit(from, to) => {
                self.move_unit(from, to);
            }
            SelectUnit(pos) => {
                assert!(self.unit(pos).is_some(),
                        "The tile for the selected unit is empty!");
                self.selected_unit = Some(pos);
            }
            Deselect => {
                assert!(self.selected_unit.is_some(),
                        "Received deselect with no unit selected");
                self.selected_unit = None;
            }
            SelectTarget(origin, pos) => {
                self.select_target(origin, pos, state, queue);
            }
            MoveUnitAndAct(origin, destination) => {
                self.move_unit_and_act(origin, destination, state, queue);
            }
            DestroyUnit(pos) => {
                let tile = self.tile_mut(pos);
                info!("Unit at {:?} destroyed! ({:?})",
                      pos,
                      tile.unit.as_ref().expect("no unit to destroy"));
                tile.unit = None;
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, _state: &State<'a>, renderer: &mut Renderer) {
        let (cols, rows) = self.size;
        let (cw, ch) = self.tile_size;
        let grid_height = rows * ch;
        for col in 0..cols {
            for row in 0..rows {
                let x = col * cw;
                let y = grid_height - ch - (row * ch);
                // sprite.render(renderer, x as i32, y as i32, Some(self.tile_size));
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
                    sprite.render(renderer, x as i32, y as i32, Some(self.tile_size));
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
