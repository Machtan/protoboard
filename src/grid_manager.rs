use std::collections::BTreeSet;
use std::fmt::{self, Debug};

use glorious::{Behavior, Color, Renderer, Sprite};
use sdl2::rect::Rect;

use common::{State, Message};
use faction::Faction;
use grid::PathFinder;
use menus::ModalMenu;
use resources::{FIRA_SANS_PATH, MARKER_PATH};
use target_selector::TargetSelector;
use terrain::Terrain;
use unit::Unit;
use unit_mover::UnitMover;

const COLOR_RED_UNIT: Color = Color(0xff, 0x66, 0x66, 0xcc);
const COLOR_RED_UNIT_SPENT: Color = Color(0x99, 0x44, 0x44, 0xcc);
const COLOR_BLUE_UNIT: Color = Color(0x66, 0xbb, 0xdd, 0xcc);
const COLOR_BLUE_UNIT_SPENT: Color = Color(65, 120, 140, 0xcc);

const COLOR_SELECTED: Color = Color(0xdd, 0xee, 0x77, 0xbb);
const COLOR_MOVEMENT_RANGE: Color = Color(0x00, 0xff, 0xff, 0x77);
const COLOR_ATTACK_RANGE: Color = Color(0xff, 0x66, 0x66, 0x77);

const COLOR_GRASS_EVEN: Color = Color(0x66, 0xcc, 0x66, 0xff);
const COLOR_GRASS_ODD: Color = Color(0x99, 0xff, 0x99, 0xff);

#[derive(Debug)]
struct Selected {
    pos: (u32, u32),
    path_finder: PathFinder,
}

#[derive(Debug)]
struct ShowingRangeOf {
    pos: (u32, u32),
    path_finder: PathFinder,
    attack_range: BTreeSet<(u32, u32)>,
}

pub struct GridManager {
    selected: Option<Selected>,
    showing_range_of: Option<ShowingRangeOf>,
    cursor: (u32, u32),
    cursor_hidden: bool,
    mouse: Option<(i32, i32)>,
}

impl GridManager {
    #[inline]
    pub fn new(cursor: (u32, u32)) -> GridManager {
        GridManager {
            selected: None,
            showing_range_of: None,
            cursor: cursor,
            cursor_hidden: false,
            mouse: None,
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
        let targets = {
            let unit = state.grid.unit(pos).expect("no unit to select");
            if pos == origin {
                state.grid
                    .find_attackable_before_moving(unit, pos)
                    .collect()
            } else {
                state.grid
                    .find_attackable_after_moving(unit, pos)
                    .collect()
            }
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
        let selected = self.selected.take().expect("no unit was selected");
        let origin = selected.pos;
        if target != origin && state.grid.unit(target).is_some() {
            // TODO: Beep!
            self.selected = Some(selected);
            return;
        }
        assert!(selected.path_finder.can_move_to(target));

        self.move_cursor_to(target, state);
        self.cursor_hidden = true;

        let unit = state.grid.remove_unit(origin);
        let mut path = selected.path_finder.random_path_rev(target).collect::<Vec<_>>();
        path.reverse();

        let mover = UnitMover::new(unit, origin, path);
        state.push_modal(Box::new(mover), queue);
    }

    /// Handles the selection of a unit.
    fn select_unit(&mut self, pos: (u32, u32), state: &mut State, _queue: &mut Vec<Message>) {
        let unit = state.grid.unit(pos).expect("cannot select unit on empty tile");
        if state.turn_info.can_act(unit) {
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
            let attack_range = path_finder.total_attack_range(&state.grid);
            self.showing_range_of = Some(ShowingRangeOf {
                pos: self.cursor,
                path_finder: path_finder,
                attack_range: attack_range,
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
        let unit = state.grid.remove_unit(pos);
        self.unit_destroyed(pos, &unit, state, queue);
    }

    fn unit_destroyed(&mut self,
                      pos: (u32, u32),
                      unit: &Unit,
                      state: &State,
                      queue: &mut Vec<Message>) {
        debug!("Unit at {:?} destroyed! ({:?})", pos, unit);
        if state.grid.units().all(|u| u.faction != unit.faction) {
            queue.push(Message::FactionDefeated(unit.faction));
        }
    }

    fn move_cursor_to(&mut self, pos: (u32, u32), state: &mut State) {
        assert!(pos.0 < state.grid.size().0 && pos.1 < state.grid.size().1);
        if let Some(ref selected) = self.selected {
            // TODO: You can move cursor to friendly unit (no crash, though).
            if !selected.path_finder.can_move_to(pos) {
                return;
            }
        }
        self.cursor = pos;
        state.ensure_in_range(pos);
    }

    #[inline]
    fn move_cursor_up(&mut self, state: &mut State) {
        let (x, y) = self.cursor;
        if y < state.grid.size().1 - 1 {
            self.move_cursor_to((x, y + 1), state);
        }
    }

    #[inline]
    fn move_cursor_down(&mut self, state: &mut State) {
        let (x, y) = self.cursor;
        if y > 0 {
            self.move_cursor_to((x, y - 1), state);
        }
    }

    #[inline]
    fn move_cursor_left(&mut self, state: &mut State) {
        let (x, y) = self.cursor;
        if x > 0 {
            self.move_cursor_to((x - 1, y), state);
        }
    }

    #[inline]
    fn move_cursor_right(&mut self, state: &mut State) {
        let (x, y) = self.cursor;
        if x < state.grid.size().0 - 1 {
            self.move_cursor_to((x + 1, y), state);
        }
    }

    fn handle_unit_moved(&mut self,
                         origin: (u32, u32),
                         target: (u32, u32),
                         state: &mut State,
                         queue: &mut Vec<Message>) {
        use common::Message::*;
        debug!("Moved unit from {:?} to {:?}", origin, target);

        let options = {
            let unit = state.grid.unit(target).expect("unreachable; failed to move unit");

            let mut options = Vec::with_capacity(2);
            let mut find_attackable = if origin == target {
                state.grid.find_attackable_before_moving(unit, target)
            } else {
                state.grid.find_attackable_after_moving(unit, target)
            };
            if find_attackable.next().is_some() {
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
                    // TODO: Marking the unit now is one frame quicker.
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
}

impl<'a> Behavior<State<'a>> for GridManager {
    type Message = Message;

    fn update(&mut self, state: &mut State<'a>, _queue: &mut Vec<Message>) {
        state.ensure_in_range(self.cursor);
        if let Some(pos) = self.mouse.and_then(|(x, y)| state.window_to_grid(x, y)) {
            self.move_cursor_to(pos, state);
        }
    }

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
                self.mouse = None;
                self.move_cursor_up(state);
            }
            MoveCursorDown => {
                self.mouse = None;
                self.move_cursor_down(state);
            }
            MoveCursorLeft => {
                self.mouse = None;
                self.move_cursor_left(state);
            }
            MoveCursorRight => {
                self.mouse = None;
                self.move_cursor_right(state);
            }

            // Modal messages
            AttackSelected(pos, target) => {
                // self.cursor.pos = target;
                self.select_target(pos, target, state, queue);
            }
            WaitSelected => {
                // self.cursor.pos = target;
                self.cursor_hidden = false;
            }
            CancelSelected(pos, target) => {
                state.grid.move_unit(target, pos);
                self.move_cursor_to(pos, state);
                self.cursor_hidden = false;
                self.select_unit(pos, state, queue);
            }
            TargetSelectorCanceled(origin, pos) => {
                self.handle_unit_moved(origin, pos, state, queue);
            }

            // State changes
            UnitSpent(pos) => {
                if let Some(unit) = state.grid.unit_mut(pos) {
                    unit.spent = true;
                }
                state.turn_info.spend_action();
            }
            UnitMoved(from, to) => {
                let (_, unit) = state.active_unit.take().expect("no active unit after move");
                state.grid.add_unit(unit, to);
                self.handle_unit_moved(from, to, state, queue);
            }
            AttackWithUnit(pos, target) => {
                self.cursor_hidden = false;

                let mut attacker = state.grid.remove_unit(pos);

                let damage = {
                    let (target_unit, terrain) = state.grid.tile(target);
                    let target_unit = target_unit.expect("no unit to attack");

                    debug!("Unit at {:?} ({:?}) attacked unit at {:?} ({:?})",
                           pos,
                           attacker,
                           target,
                           target_unit);

                    attacker.attack_damage(target_unit, terrain)
                };

                let target_destroyed =
                    state.grid.unit_mut(target).expect("no unit to attack").receive_damage(damage);
                let attacker_destroyed = if target_destroyed {
                    self.destroy_unit(target, state, queue);
                    false
                } else {
                    let defender = state.grid.unit(target).expect("no unit for counter-attack");
                    let in_range = state.grid
                        .attack_range_when_retaliating(defender, target)
                        .any(|p| p == pos);
                    if in_range {
                        let terrain = state.grid.terrain(pos);
                        let damage = defender.retaliation_damage(damage, &attacker, terrain);
                        attacker.receive_damage(damage)
                    } else {
                        false
                    }
                };

                if attacker_destroyed {
                    self.unit_destroyed(pos, &attacker, state, queue);
                } else {
                    state.grid.add_unit(attacker, pos);
                }
            }
            FinishTurn => {
                self.selected = None;
                for unit in state.grid.units_mut() {
                    unit.spent = false;
                }
                state.turn_info.end_turn();
                // TODO: Display a turn change animation here
            }

            MouseMovedTo(x, y) |
            LeftClickAt(x, y) |
            RightClickAt(x, y) => {
                self.mouse = Some((x, y));
                let pos = match state.window_to_grid(x, y) {
                    Some(pos) => pos,
                    None => return,
                };
                self.move_cursor_to(pos, state);
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

            FactionDefeated(faction) => {
                info!("Faction defeated! {:?}", faction);
                state.turn_info.remove_faction(faction);

                let (&faction, rest) = state.turn_info
                    .factions()
                    .split_first()
                    .expect("there must be at least one faction left");
                // TODO: Alliances? Neutrals?
                if rest.iter().all(|&f| f == faction) {
                    queue.push(FactionWins(faction));
                }
            }
            FactionWins(faction) => {
                info!("Faction won! ({:?})", faction);
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
                    renderer.set_draw_color(COLOR_GRASS_EVEN);
                } else {
                    renderer.set_draw_color(COLOR_GRASS_ODD);
                }
                renderer.fill_rect(rect).unwrap();

                let texture_path = match *terrain {
                    Terrain::Grass => None,
                    Terrain::Mountains => Some("assets/mountains.png"),
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
                            Some(COLOR_SELECTED)
                        } else if s.path_finder.can_move_to(pos) {
                            if unit.is_none() {
                                Some(COLOR_MOVEMENT_RANGE)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    });

                if let Some(color) = color {
                    renderer.set_draw_color(color);
                    renderer.fill_rect(rect).unwrap();
                }

                if let Some(unit) = unit {
                    render_unit(unit, rect, color.is_none(), state, renderer);
                }

                if let Some((active_pos, ref unit)) = state.active_unit {
                    if active_pos == pos {
                        render_unit(unit, rect, true, state, renderer);
                    }
                }

                if let Some(ref sro) = self.showing_range_of {
                    if sro.pos != pos && sro.attack_range.contains(&pos) {
                        renderer.set_draw_color(COLOR_ATTACK_RANGE);
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

pub fn render_unit(unit: &Unit, rect: Rect, _bg: bool, state: &State, renderer: &mut Renderer) {
    let color = if unit.spent {
        match unit.faction {
            Faction::Red => COLOR_RED_UNIT_SPENT,
            Faction::Blue => COLOR_BLUE_UNIT_SPENT,
        }
    } else {
        match unit.faction {
            Faction::Red => COLOR_RED_UNIT,
            Faction::Blue => COLOR_BLUE_UNIT,
        }
    };
    let sprite = Sprite::new(unit.texture(), None);
    sprite.render_rect(renderer, rect);

    let label = state.health_label(unit.health);

    let hw = rect.width() / 2;
    let hh = rect.height() / 2;
    let box_rect = Rect::new(rect.x() + hw as i32 + 4,
                             rect.y() + hh as i32 + 7,
                             hw - 8,
                             hh - 11);
    renderer.set_draw_color(color);
    renderer.fill_rect(box_rect).unwrap();

    let (lw, lh) = label.size();

    let lx = box_rect.x() + (box_rect.width() as i32 - lw as i32) / 2;
    let ly = rect.y() + hh as i32 + 8 - (lh * 2 / 15) as i32;

    label.render(renderer, lx, ly);
}

impl Debug for GridManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GridManager")
            .field("selected", &self.selected)
            .field("showing_range_of", &self.showing_range_of)
            .field("cursor", &self.cursor)
            .field("cursor_hidden", &self.cursor)
            .finish()
    }
}
