use std::collections::BTreeSet;
use std::fmt::{self, Debug};
use std::time::Duration;

use glorious::{Behavior, Label, Renderer, Sprite};
use lru_time_cache::LruCache;
use sdl2::pixels::Color;

use common::{State, Message};
use faction::Faction;
use grid::{PathFinder, Terrain};
use menus::ModalMenu;
use resources::{FIRA_SANS_PATH, FIRA_SANS_BOLD_PATH, MARKER_PATH};
use target_selector::TargetSelector;

#[derive(Debug)]
struct Selected {
    pos: (u32, u32),
    path_finder: PathFinder,
}

#[derive(Debug)]
struct ShowingRangeOf {
    pos: (u32, u32),
    path_finder: PathFinder,
    attackable: BTreeSet<(u32, u32)>,
}

pub struct GridManager {
    selected: Option<Selected>,
    showing_range_of: Option<ShowingRangeOf>,
    cursor: (u32, u32),
    cursor_hidden: bool,
    health_labels: LruCache<u32, Label>,
}

impl GridManager {
    pub fn new(cursor: (u32, u32)) -> GridManager {
        let expiry_duration = Duration::from_millis(100);
        GridManager {
            selected: None,
            showing_range_of: None,
            cursor: cursor,
            cursor_hidden: false,
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
        debug!("Selecting target...");
        assert!(self.selected.is_some());
        let targets = {
            let unit = state.grid.unit(pos).expect("no unit to select");
            state.grid
                .find_attackable(unit, pos)
                .map(|(pos, _)| pos)
                .collect()
        };
        let selector = TargetSelector::new(pos, origin, targets);
        self.cursor_hidden = true;
        state.push_modal(Box::new(selector), queue);
    }

    /// Moves the selected unit from origin to target and opens up the action menu.
    /// If the menu is cancelled, the unit moves back.
    fn move_selected_unit_and_act<'a>(&mut self,
                                      target: (u32, u32),
                                      state: &mut State<'a>,
                                      queue: &mut Vec<Message>) {
        use common::Message::*;

        let origin = {
            let selected = self.selected.as_ref().expect("no unit was selected");

            if target != selected.pos && state.grid.unit(target).is_some() {
                // TODO: Beep!
                return;
            }
            assert!(selected.path_finder.costs.contains_key(&target));
            selected.pos
        };

        state.grid.move_unit(origin, target);

        debug!("Moved unit from {:?} to {:?}", origin, target);

        self.move_cursor_to(target, state.grid.size());
        self.cursor_hidden = true;

        let options = {
            let unit = state.grid.unit(target).expect("unreachable; failed to move unit");

            let mut options = Vec::with_capacity(2);
            let can_attack = if unit.is_ranged() {
                origin == target
            } else {
                true
            };
            if can_attack && state.grid.find_attackable(unit, target).next().is_some() {
                options.push("Attack");
            }
            options.push("Wait");
            options
        };

        // Clicking the unit confirms too :)
        let extra_confirm_areas = vec![state.tile_rect(target)];

        let menu = ModalMenu::new(options.iter().map(|&s| s.to_owned()),
                                  0,
                                  (50, 50),
                                  state.resources.font(FIRA_SANS_PATH, 16),
                                  state,
                                  extra_confirm_areas,
                                  move |option, state, queue| {
            match option {
                Some("Attack") => {
                    debug!("Attack!");
                    state.pop_modal(queue);
                    queue.push(AttackSelected(origin, target));
                }
                Some("Wait") => {
                    debug!("Wait!");
                    state.pop_modal(queue);
                    // TODO
                    queue.push(UnitSpent(target));
                    queue.push(WaitSelected);
                }
                None => {
                    debug!("Cancel!");
                    state.pop_modal(queue);
                    queue.push(CancelSelected(origin, target));
                }
                _ => unreachable!(),
            }
        })
            .expect("could not create menu");
        state.push_modal(Box::new(menu), queue);
    }

    /// Handles the selection of a unit.
    fn select_unit(&mut self, pos: (u32, u32), state: &mut State, _queue: &mut Vec<Message>) {
        let unit = state.grid.unit(pos).expect("cannot select unit on empty tile");
        if state.actions_left > 0 && unit.faction == state.current_turn && !unit.spent {
            debug!("Unit at {:?} selected!", pos);
            let path_finder = state.grid.path_finder(pos);
            self.selected = Some(Selected {
                pos: pos,
                path_finder: path_finder,
            });
        }
    }

    /// Handles a confirm press at the given target tile when a unit is selected.
    fn confirm(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        let target = self.cursor;
        if self.selected.is_some() {
            self.move_selected_unit_and_act(target, state, queue);
        } else if state.grid.unit(target).is_some() {
            self.select_unit(target, state, queue);
        }
    }

    /// Handles a cancel press at the given position.
    fn cancel<'a>(&mut self, state: &State<'a>, _queue: &mut Vec<Message>) {
        if self.selected.is_some() {
            self.selected = None;
        } else if state.grid.unit(self.cursor).is_some() {
            let path_finder = state.grid.path_finder(self.cursor);
            let attackable = path_finder.tiles_in_attack_range(&state.grid);
            self.showing_range_of = Some(ShowingRangeOf {
                pos: self.cursor,
                path_finder: path_finder,
                attackable: attackable,
            });
        }
    }

    /// Handles the release of the cancel button.
    fn cancel_release(&mut self) {
        self.showing_range_of = None;
    }

    /// Destroys the unit on the given tile.
    fn destroy_unit<'a>(&mut self,
                        pos: (u32, u32),
                        state: &mut State<'a>,
                        queue: &mut Vec<Message>) {
        let faction = {
            let faction = {
                let unit = state.grid.unit(pos).expect("no unit to destroy");
                debug!("Unit at {:?} destroyed! ({:?})", pos, unit);
                unit.faction
            };
            state.grid.remove_unit(pos);
            faction
        };
        if state.grid.units().all(|u| u.faction != faction) {
            queue.push(Message::FactionDefeated(faction));
        }
    }

    fn deselect(&mut self) {
        assert!(self.selected.is_some(),
                "received deselect with no unit selected");
        self.selected = None;
        self.cursor_hidden = false;
    }

    fn move_cursor_to(&mut self, pos: (u32, u32), size: (u32, u32)) {
        assert!(pos.0 < size.0 && pos.1 < size.1);
        if let Some(ref selected) = self.selected {
            if !selected.path_finder.costs.contains_key(&pos) {
                return;
            }
        }
        self.cursor = pos;
    }

    #[inline]
    fn move_cursor_up(&mut self, size: (u32, u32)) {
        let (x, y) = self.cursor;
        if y < size.1 - 1 {
            self.move_cursor_to((x, y + 1), size);
        }
    }

    #[inline]
    fn move_cursor_down(&mut self, size: (u32, u32)) {
        let (x, y) = self.cursor;
        if y > 0 {
            self.move_cursor_to((x, y - 1), size);
        }
    }

    #[inline]
    fn move_cursor_left(&mut self, size: (u32, u32)) {
        let (x, y) = self.cursor;
        if x > 0 {
            self.move_cursor_to((x - 1, y), size);
        }
    }

    #[inline]
    fn move_cursor_right(&mut self, size: (u32, u32)) {
        let (x, y) = self.cursor;
        if x < size.0 - 1 {
            self.move_cursor_to((x + 1, y), size);
        }
    }
}

impl<'a> Behavior<State<'a>> for GridManager {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            // Input
            Confirm => {
                self.confirm(state, queue);
            }
            Cancel => {
                self.cancel(state, queue);
            }
            RightReleasedAt(_, _) |
            CancelReleased => {
                self.cancel_release();
            }
            MoveCursorUp => {
                self.move_cursor_up(state.grid.size());
            }
            MoveCursorDown => {
                self.move_cursor_down(state.grid.size());
            }
            MoveCursorLeft => {
                self.move_cursor_left(state.grid.size());
            }
            MoveCursorRight => {
                self.move_cursor_right(state.grid.size());
            }

            // Modal messages
            AttackSelected(pos, target) => {
                // self.cursor.pos = target;
                self.select_target(pos, target, state, queue);
            }
            WaitSelected => {
                // self.cursor.pos = target;
                self.deselect();
                self.cursor_hidden = false;
            }
            CancelSelected(pos, target) => {
                state.grid.move_unit(target, pos);
                self.move_cursor_to(pos, state.grid.size());
                self.cursor_hidden = false;
                self.select_unit(pos, state, queue);
            }

            TargetSelectorCanceled(pos) => {
                let origin = self.selected.as_ref().expect("a unit must be selected here!").pos;
                // TODO: We cancel, just to move back.
                state.grid.move_unit(pos, origin);
                self.move_selected_unit_and_act(pos, state, queue);
            }

            // State changes
            UnitSpent(pos) => {
                state.grid
                    .unit_mut(pos)
                    .expect("no unit to mark as spent")
                    .spent = true;
            }
            Deselect => {
                self.deselect();
            }
            AttackWithUnit(pos, target) => {
                let destroyed = {
                    // TODO: Have target not borrow attacker.
                    let (attacker, target_unit) =
                        state.grid.unit_pair_mut(pos, target).expect("a unit cannot attack itself");

                    debug!("Unit at {:?} ({:?}) attacked unit at {:?} ({:?})",
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
                    self.destroy_unit(target, state, queue);
                }
            }
            FinishTurn => {
                for unit in state.grid.units_mut() {
                    unit.spent = false;
                }
            }

            MouseMovedTo(x, y) |
            LeftClickAt(x, y) |
            RightClickAt(x, y) => {
                let pos = state.window_to_grid(x, y);
                self.move_cursor_to(pos, state.grid.size());
                match message {
                    MouseMovedTo(..) => {}
                    LeftClickAt(..) => {
                        self.confirm(state, queue);
                    }
                    RightClickAt(..) => {
                        self.cancel(state, queue);
                    }
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        let (cols, rows) = state.grid.size();
        for col in 0..cols {
            for row in 0..rows {
                let pos = (col, row);

                let rect = state.tile_rect(pos);
                let (unit, terrain) = state.grid.tile(pos);

                if (col + row) % 2 == 0 {
                    renderer.set_draw_color(Color::RGB(110, 210, 110));
                } else {
                    renderer.set_draw_color(Color::RGB(155, 255, 155));
                }
                // TODO: When can `fill_rect` fail?
                renderer.fill_rect(rect).unwrap();

                let texture_path = match *terrain {
                    Terrain::Grass => None,
                    Terrain::Mountain => Some("assets/mountains.png"),
                    Terrain::Woods => Some("assets/woods.png"),
                };
                if let Some(path) = texture_path {
                    let texture = state.resources.texture(path);
                    let sprite = Sprite::new(texture, None);
                    sprite.render_rect(renderer, rect);
                }

                let color = self.selected
                    .as_ref()
                    .and_then(|s| {
                        if s.pos == pos {
                            Some(Color::RGBA(244, 237, 129, 191))
                        } else if s.path_finder.costs.contains_key(&pos) {
                            Some(Color::RGBA(0, 255, 255, 127))
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        unit.as_ref().and_then(|u| {
                            if u.spent {
                                match u.faction {
                                    Faction::Red => Some(Color::RGBA(150, 65, 65, 127)),
                                    Faction::Blue => Some(Color::RGBA(65, 120, 140, 127)),
                                }
                            } else {
                                match u.faction {
                                    Faction::Red => Some(Color::RGBA(255, 100, 100, 127)),
                                    Faction::Blue => Some(Color::RGBA(100, 180, 220, 127)),
                                }
                            }
                        })
                    });

                if let Some(color) = color {
                    renderer.set_draw_color(color);
                    renderer.fill_rect(rect).unwrap();
                }

                if let Some(unit) = unit {
                    let sprite = Sprite::new(unit.texture(), None);
                    sprite.render_rect(renderer, rect);

                    let font = state.resources.font(FIRA_SANS_BOLD_PATH, 16);
                    let (_, sy) = renderer.device().scale();
                    // TODO: Maybe we should wrap `Font` in glorious to automatically scale?
                    let descent = (font.descent() as f32 / sy) as i32;
                    let height = (font.height() as f32 / sy) as i32;

                    let label = self.health_labels.entry(unit.health).or_insert_with(|| {
                        let string = format!("{}", unit.health);
                        Label::new(&font, &string, (255, 255, 255, 255), renderer.device())
                    });
                    let (w, _) = label.size();

                    let lx = rect.x() + rect.width() as i32 - 3 - w as i32;
                    let ly = rect.y() + rect.height() as i32 - 3 - height - descent;

                    label.render(renderer, lx, ly);
                }

                if let Some(ref showing_range_of) = self.showing_range_of {
                    if showing_range_of.attackable.contains(&pos) {
                        renderer.set_draw_color(Color::RGBA(255, 100, 100, 127));
                        renderer.fill_rect(rect).unwrap();
                    }
                }
                if self.cursor == (col, row) && !self.cursor_hidden {
                    let sprite = Sprite::new(state.resources.texture(MARKER_PATH), None);
                    sprite.render_rect(renderer, rect);
                }
            }
        }
    }
}

impl Debug for GridManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GridManager")
            .field("selected", &self.selected)
            .field("showing_range_of", &self.showing_range_of)
            .field("cursor", &self.cursor)
            .field("cursor_hidden", &self.cursor)
            .field("health_labels", &(..))
            .finish()
    }
}
