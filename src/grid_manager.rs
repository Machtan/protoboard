use std::fmt::{self, Debug};
use std::time::Duration;

use glorious::{Behavior, Label, Renderer, Sprite};
use lru_time_cache::LruCache;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use common::{State, Message};
use faction::Faction;
use grid::{Grid, Terrain};
use menus::ModalMenu;
use resources::{FIRA_SANS_PATH, FIRA_SANS_BOLD_PATH};
use target_selector::TargetSelector;

pub struct GridManager {
    grid: Grid,
    tile_size: (u32, u32),
    selected: Option<(u32, u32)>,
    showing_range_of: Option<(u32, u32)>,
    health_labels: LruCache<u32, Label>,
}

impl GridManager {
    #[inline]
    pub fn new(grid: Grid, tile_size: (u32, u32)) -> GridManager {
        let expiry_duration = Duration::from_millis(100);
        GridManager {
            grid: grid,
            tile_size: tile_size,
            selected: None,
            showing_range_of: None,
            health_labels: LruCache::with_expiry_duration(expiry_duration),
        }
    }

    /// Opens the target selection modal for the unit at Cell.
    /// The origin is used to return to the menu when cancelling.
    fn select_target<'a>(&mut self,
                         origin: (u32, u32),
                         pos: (u32, u32),
                         state: &mut State<'a>,
                         queue: &mut Vec<Message>) {
        info!("Selecting target...");
        let unit = self.grid.unit(pos).expect("no unit to select");
        let targets = self.grid
            .find_attackable(unit, pos)
            .map(|(pos, _)| pos)
            .collect();
        let selector = TargetSelector::new(pos, origin, self.grid.size(), self.tile_size, targets);
        queue.push(Message::HideCursor);
        state.push_modal(Box::new(selector), queue);
    }

    /// Moves the selected unit from origin to target and opens up the action menu.
    /// If the menu is cancelled, the unit moves back.
    fn move_unit_and_act<'a>(&mut self,
                             origin: (u32, u32),
                             target: (u32, u32),
                             state: &mut State<'a>,
                             queue: &mut Vec<Message>) {
        use common::Message::*;

        assert!(self.selected == Some(origin),
                "the moved unit is not selected");

        if target != origin && self.grid.unit(target).is_some() {
            // TODO: Beep!
            return;
        }

        self.grid.move_unit(origin, target);

        debug!("Moved unit from {:?} to {:?}", origin, target);

        let unit = self.grid.unit(target).expect("unreachable; failed to move unit");

        let mut options = Vec::with_capacity(2);
        let can_attack = if unit.is_ranged() {
            origin == target
        } else {
            true
        };
        if can_attack && self.grid.find_attackable(unit, target).next().is_some() {
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

    /// Handles the selection of a unit.
    fn select_unit(&mut self, pos: (u32, u32), state: &mut State, _queue: &mut Vec<Message>) {
        let unit = self.grid.unit(pos).expect("cannot select unit on empty tile");
        if state.actions_left > 0 && unit.faction == state.current_turn && !unit.spent {
            info!("Unit at {:?} selected!", pos);
            self.selected = Some(pos);
        }
    }

    /// Handles a confirm press at the given target tile when a unit is selected.
    fn confirm(&mut self, target: (u32, u32), state: &mut State, queue: &mut Vec<Message>) {
        match self.selected {
            Some(origin) => {
                self.move_unit_and_act(origin, target, state, queue);
            }
            None => {
                if self.grid.unit(target).is_some() {
                    self.select_unit(target, state, queue);
                }
            }
        }
    }

    /// Handles a cancel press at the given position.
    fn cancel(&mut self, cursor_pos: (u32, u32)) {
        if self.selected.is_some() {
            self.selected = None;
        } else if self.grid.unit(cursor_pos).is_some() {
            self.showing_range_of = Some(cursor_pos);
        }
    }

    /// Handles the release of the cancel button.
    fn cancel_release(&mut self) {
        if self.showing_range_of.is_some() {
            self.showing_range_of = None;
        }
    }

    /// Destroys the unit on the given tile.
    fn destroy_unit(&mut self, pos: (u32, u32), queue: &mut Vec<Message>) {
        let faction = {
            let faction = {
                let unit = self.grid.unit(pos).expect("no unit to destroy");
                info!("Unit at {:?} destroyed! ({:?})", pos, unit);
                unit.faction
            };
            self.grid.remove_unit(pos);
            faction
        };
        if self.grid.units().all(|u| u.faction != faction) {
            queue.push(Message::FactionDefeated(faction));
        }
    }

    /// Returns the grid position of a window position.
    fn window_to_grid(&self, x: i32, y: i32) -> (u32, u32) {
        assert!(x >= 0 && y >= 0);
        let (w, h) = self.tile_size;
        let (_, rows) = self.grid.size();
        let col = (x as u32 - (x as u32 % w)) / w;
        let row = rows - 1 - (y as u32 - (y as u32 % h)) / h;
        (col, row)
    }
}

impl<'a> Behavior<State<'a>> for GridManager {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            CursorConfirm(pos) => {
                self.confirm(pos, state, queue);
            }
            LeftClickAt(x, y) => {
                let pos = self.window_to_grid(x, y);
                self.confirm(pos, state, queue);
            }
            RightClickAt(x, y) => {
                let pos = self.window_to_grid(x, y);
                self.cancel(pos);
            }
            RightReleasedAt(_, _) => {
                self.cancel_release();
            }
            CursorCancel(pos) => {
                self.cancel(pos);
            }
            CancelReleased => {
                self.cancel_release();
            }
            UnitSpent(pos) => {
                self.grid
                    .unit_mut(pos)
                    .expect("no unit to mark as spent")
                    .spent = true;
            }
            MoveUnit(from, to) => {
                self.grid.move_unit(from, to);
            }
            SelectUnit(pos) => {
                self.select_unit(pos, state, queue);
            }
            Deselect => {
                assert!(self.selected.is_some(),
                        "received deselect with no unit selected");
                self.selected = None;
            }
            SelectTarget(origin, pos) => {
                self.select_target(origin, pos, state, queue);
            }
            MoveUnitAndAct(origin, destination) => {
                self.move_unit_and_act(origin, destination, state, queue);
            }
            AttackWithUnit(pos, target) => {
                let destroyed = {
                    // TODO: Have target not borrow attacker.
                    let (attacker, target_unit) =
                        self.grid.unit_pair_mut(pos, target).expect("a unit cannot attack itself");

                    info!("Unit at {:?} ({:?}) attacked unit at {:?} ({:?})",
                          pos,
                          attacker,
                          target,
                          target_unit);

                    let attacker = attacker.expect("no attacking unit");
                    let target_unit = target_unit.expect("no unit to attack");

                    // TODO: This call would need terrain information.
                    target_unit.receive_attack(attacker)
                };
                if destroyed {
                    self.destroy_unit(target, queue);
                }
            }
            FinishTurn => {
                for unit in self.grid.units_mut() {
                    unit.spent = false;
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        let (cols, rows) = self.grid.size();
        let (cw, ch) = self.tile_size;
        let grid_height = rows * ch;
        for col in 0..cols {
            for row in 0..rows {
                let x = col * cw;
                let y = grid_height - ch - (row * ch);
                let rect = Rect::new(x as i32, y as i32, cw, ch);

                let (unit, terrain) = self.grid.tile((col, row));

                match *terrain {
                    Terrain::Grass => {
                        if (col + row) % 2 == 0 {
                            renderer.set_draw_color(Color::RGB(110, 210, 110));
                        } else {
                            renderer.set_draw_color(Color::RGB(155, 255, 155));
                        }
                        // TODO: When can `fill_rect` fail?
                        renderer.fill_rect(rect).unwrap();
                    }
                }

                if let Some(unit) = unit {
                    let color = if self.selected == Some((col, row)) {
                        Color::RGB(244, 237, 129)
                    } else if unit.spent {
                        match unit.faction {
                            Faction::Red => Color::RGB(150, 43, 43),
                            Faction::Blue => Color::RGB(65, 120, 140),
                        }
                    } else {
                        match unit.faction {
                            Faction::Red => Color::RGB(220, 100, 100),
                            Faction::Blue => Color::RGB(100, 180, 220),
                        }
                    };

                    renderer.set_draw_color(color);
                    renderer.fill_rect(rect).unwrap();
                    let sprite = Sprite::new(unit.texture(), None);
                    sprite.render(renderer, x as i32, y as i32, Some(self.tile_size));

                    let font = state.resources.font(FIRA_SANS_BOLD_PATH, 16);
                    let (_, sy) = renderer.scale();
                    // TODO: Maybe we should wrap `Font` in glorious to automatically scale?
                    let descent = (font.descent() as f32 / sy) as i32;
                    let height = (font.height() as f32 / sy) as i32;

                    let label = self.health_labels.entry(unit.health).or_insert_with(|| {
                        let string = format!("{}", unit.health);
                        Label::new(font, string, (255, 255, 255, 255), renderer.clone())
                    });
                    let (w, _) = label.size();

                    let lx = x as i32 + cw as i32 - 3 - w as i32;
                    let ly = y as i32 + ch as i32 - 3 - height - descent;

                    label.render(renderer, lx, ly);
                }
            }
        }

        if let Some(pos) = self.showing_range_of {
            let target_color = Color::RGB(252, 223, 80);
            renderer.set_draw_color(target_color);
            for (col, row) in self.grid.tiles_in_range(pos) {
                let x = col * cw;
                let y = grid_height - ch - (row * ch);
                let rect = Rect::new(x as i32, y as i32, cw, ch);
                renderer.fill_rect(rect).unwrap();

                let (unit, _) = self.grid.tile((col, row));
                if let Some(unit) = unit {
                    let sprite = Sprite::new(unit.texture(), None);
                    sprite.render(renderer, x as i32, y as i32, Some(self.tile_size));
                }
            }
        }
    }
}

impl Debug for GridManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GridManager")
            .field("tile_size", &self.tile_size)
            .field("selected", &self.selected)
            .field("showing_range_of", &self.showing_range_of)
            .field("health_labels", &(..))
            .finish()
    }
}
