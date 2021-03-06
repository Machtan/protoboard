use std::collections::BTreeSet;

use glorious::{Color, Renderer, Sprite};
use sdl2::rect::Rect;

use common::{State, Message, ModalBox};
use faction::Faction;
use grid::PathFinder;
use menus::ModalMenu;
use resources::{FIRA_SANS_PATH, MARKER_PATH};
use target_selector::TargetSelector;
use unit::Unit;
use unit_mover::UnitMover;

const COLOR_RED_UNIT: Color = Color(0xff, 0x66, 0x66, 0xcc);
const COLOR_RED_UNIT_SPENT: Color = Color(0x99, 0x44, 0x44, 0xcc);
const COLOR_BLUE_UNIT: Color = Color(0x66, 0xbb, 0xdd, 0xcc);
const COLOR_BLUE_UNIT_SPENT: Color = Color(65, 120, 140, 0xcc);

const COLOR_SELECTED: Color = Color(0xdd, 0xee, 0x77, 0xbb);
const COLOR_MOVEMENT_RANGE: Color = Color(0x00, 0xff, 0xff, 0x77);
const COLOR_ATTACK_RANGE: Color = Color(0xff, 0x66, 0x66, 0x77);

const COLOR_DEFAULT_EVEN: Color = Color(0xcc, 0xcc, 0xcc, 0xff);
const COLOR_DEFAULT_ODD: Color = Color(0xdd, 0xdd, 0xdd, 0xff);

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

#[derive(Debug)]
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

    #[inline]
    pub fn hide_cursor(&mut self) {
        self.cursor_hidden = false;
    }

    #[inline]
    pub fn deselect(&mut self) {
        self.selected = None;
    }

    /// Opens the target selection modal for the unit at Cell.
    /// The origin is used to return to the menu when cancelling.
    pub fn select_target(&mut self,
                         origin: (u32, u32),
                         pos: (u32, u32),
                         state: &mut State)
                         -> ModalBox {
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
        self.cursor_hidden = true;
        Box::new(TargetSelector::new(pos, origin, targets))
    }

    /// Moves the selected unit from origin to target and opens up the action menu.
    /// If the menu is cancelled, the unit moves back.
    fn move_selected_unit_and_act(&mut self,
                                  target: (u32, u32),
                                  state: &mut State)
                                  -> Option<ModalBox> {
        let selected = self.selected.take().expect("no unit was selected");
        let origin = selected.pos;
        if target != origin && state.grid.unit(target).is_some() {
            // TODO: Beep!
            self.selected = Some(selected);
            return None;
        }
        assert!(selected.path_finder.can_move_to(target));

        self.move_cursor_to(target, state);
        self.cursor_hidden = true;

        let unit = state.grid.remove_unit(origin);
        let mut path = selected.path_finder.random_path_rev(target).collect::<Vec<_>>();
        path.reverse();
        Some(Box::new(UnitMover::new(unit, origin, path)))
    }

    /// Handles the selection of a unit.
    pub fn select_unit(&mut self, pos: (u32, u32), state: &mut State) {
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
    pub fn confirm(&mut self, state: &mut State) -> Option<ModalBox> {
        let target = self.cursor;
        if self.selected.is_some() {
            self.move_selected_unit_and_act(target, state)
        } else {
            if state.grid.unit(target).is_some() {
                self.select_unit(target, state);
            }
            None
        }
    }

    /// Handles a cancel press at the given position.
    pub fn cancel(&mut self, state: &State) {
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
    pub fn cancel_release(&mut self) {
        self.showing_range_of = None;
    }

    /// Destroys the unit on the given tile.
    fn destroy_unit(&self, pos: (u32, u32), state: &mut State) {
        state.grid.remove_unit(pos);
    }

    fn apply_damage(&self, pos: (u32, u32), damage: f64, state: &mut State) -> bool {
        if state.grid.unit_mut(pos).expect("no unit to apply damage to").receive_damage(damage) {
            self.destroy_unit(pos, state);
            true
        } else {
            false
        }
    }

    fn calculate_damage(&self,
                        pos: (u32, u32),
                        target: (u32, u32),
                        retaliating_to: Option<f64>,
                        state: &State)
                        -> f64 {
        let attacker = state.grid.unit(pos).expect("no attacking unit");
        let (defender, tile) = state.grid.unit_and_tile(target);
        let defender = defender.expect("no unit to attack");
        match retaliating_to {
            Some(damage) => attacker.retaliation_damage(damage, defender, &tile.terrain),
            None => attacker.attack_damage(defender, &tile.terrain),
        }
    }

    pub fn target_confirmed(&mut self, pos: (u32, u32), target: (u32, u32), state: &mut State) {
        self.cursor_hidden = false;

        let damage = self.calculate_damage(pos, target, None, state);
        if self.apply_damage(target, damage, state) {
            // Destroyed defender cannot retaliate.
            return;
        }
        let in_range = {
            let attacker = state.grid.unit(target).expect("no retaliating unit");
            state.grid
                .attack_range_when_retaliating(attacker, target)
                .any(|p| p == pos)
        };
        if in_range {
            let damage = self.calculate_damage(pos, target, Some(damage), state);
            self.apply_damage(pos, damage, state);
        }
    }

    pub fn move_cursor_to(&mut self, pos: (u32, u32), state: &mut State) {
        assert!(pos.0 < state.grid.size().0 && pos.1 < state.grid.size().1);
        if let Some(ref selected) = self.selected {
            // TODO: You can move cursor to friendly unit (no crash, though).
            // Note, that it is important to be able to move the cursor
            // *through* friendly units, since certain valid paths
            // would be impossible to selecte, otherwise.
            if !selected.path_finder.can_move_to(pos) {
                return;
            }
        }
        self.cursor = pos;
        state.ensure_in_range(pos);
    }

    pub fn move_cursor_relative(&mut self, delta: (i32, i32), state: &mut State) {
        self.mouse = None;

        let (w, h) = state.grid.size();
        let x = self.cursor.0 as i32 + delta.0;
        let y = self.cursor.1 as i32 + delta.1;

        if 0 <= x && x < w as i32 && 0 <= y && y < h as i32 {
            self.move_cursor_to((x as u32, y as u32), state);
        }
    }

    pub fn mouse_moved_to(&mut self, x: i32, y: i32, state: &mut State) {
        self.mouse = Some((x, y));
        let pos = match state.window_to_grid(x, y) {
            Some(pos) => pos,
            None => return,
        };
        self.move_cursor_to(pos, state);
    }

    pub fn handle_unit_moved(&mut self,
                             origin: (u32, u32),
                             target: (u32, u32),
                             state: &mut State)
                             -> ModalBox {
        debug!("Moved unit from {:?} to {:?}", origin, target);

        let options = {
            let (unit, tile) = state.grid.unit_and_tile(target);
            let unit = unit.expect("unreachable; failed to move unit");

            let mut options = Vec::with_capacity(2);
            let mut find_attackable = if origin == target {
                state.grid.find_attackable_before_moving(unit, target)
            } else {
                state.grid.find_attackable_after_moving(unit, target)
            };
            if find_attackable.next().is_some() {
                options.push("Attack");
            }
            if unit.kind.capture > 0 && tile.can_be_captured() &&
               tile.faction != Some(unit.faction) {
                options.push("Capture");
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
                    queue.push(Message::AttackSelected(origin, target));
                }
                Some("Capture") => {
                    debug!("Capture!");
                    state.pop_modal(queue);
                    queue.push(Message::CaptureSelected(target));
                }
                Some("Wait") => {
                    debug!("Wait!");
                    state.pop_modal(queue);
                    // TODO: Marking the unit now is one frame quicker.
                    queue.push(Message::UnitSpent(target));
                    queue.push(Message::WaitSelected);
                }
                None => {
                    debug!("Cancel!");
                    state.pop_modal(queue);
                    queue.push(Message::CancelSelected(origin, target));
                }
                _ => unreachable!(),
            }
        })
            .expect("could not create menu");
        Box::new(menu)
    }

    pub fn capture_at(&mut self, pos: (u32, u32), state: &mut State) {
        self.cursor_hidden = false;
        {
            let (unit, tile) = state.grid.unit_and_tile_mut(pos);
            let unit = unit.expect("no unit to capture with");
            assert!(tile.can_be_captured() && tile.faction != Some(unit.faction));

            let capture = unit.kind.capture * unit.health / 10;
            info!("Trying to capture {:?} with strength {:?}", tile.terrain.name, capture);
            if tile.capture(unit.faction, capture) {
                info!("Tile at {:?} captured by {:?}!", pos, unit.faction);
            }
        }
        self.unit_spent(pos, state);
    }

    pub fn unit_spent(&mut self, pos: (u32, u32), state: &mut State) {
        if let Some(unit) = state.grid.unit_mut(pos) {
            unit.spent = true;
        }
        state.turn_info.spend_action();
    }

    pub fn update(&mut self, state: &mut State) {
        state.ensure_in_range(self.cursor);
        if let Some(pos) = self.mouse.and_then(|(x, y)| state.window_to_grid(x, y)) {
            self.move_cursor_to(pos, state);
        }
    }

    /// Renders the object.
    pub fn render(&mut self, state: &State, renderer: &mut Renderer) {
        let (cols, rows) = state.grid.size();
        for col in 0..cols {
            for row in 0..rows {
                let pos = (col, row);

                let rect = state.tile_rect(pos);
                let (unit, tile) = state.grid.unit_and_tile(pos);

                if (col + row) % 2 == 0 {
                    renderer.set_draw_color(COLOR_DEFAULT_EVEN);
                } else {
                    renderer.set_draw_color(COLOR_DEFAULT_ODD);
                }
                renderer.fill_rect(rect).unwrap();

                if let Some(ref sprite) = tile.terrain.sprite {
                    let sprite = state.sprite(sprite);
                    sprite.render_rect(renderer, rect);
                }
                if let Some(owner) = tile.faction {
                    // TODO: Show this in a different way.
                    let color = match owner {
                        Faction::Red => Color(0xff, 0x00, 0x00, 0x33),
                        Faction::Blue => Color(0x00, 0x00, 0xff, 0x33),
                    };
                    renderer.set_draw_color(color);
                    renderer.fill_rect(rect).unwrap();
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
    let sprite = state.unit_sprite(unit);
    sprite.render_rect(renderer, rect);

    let label = state.health_label(unit.health);

    let hw = rect.width() / 2;
    let hh = rect.height() / 2;
    let box_rect = Rect::new(rect.x() + hw as i32 + 3,
                             rect.y() + hh as i32 + 8,
                             hw - 6,
                             hh - 11);
    renderer.set_draw_color(color);
    renderer.fill_rect(box_rect).unwrap();

    let (lw, _) = label.size();

    let lx = box_rect.x() + (box_rect.width() as i32 - lw as i32) / 2;
    let ly = rect.y() + hh as i32 + 5;

    label.render(renderer, lx, ly);
}
