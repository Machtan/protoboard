use glorious::{Behavior, Renderer};

use common::{ModalBox, Message, State};
use grid_manager::GridManager;
use info_box::InfoBox;
use resources::FIRA_SANS_PATH;

#[derive(Debug)]
pub struct Scene {
    grid_manager: GridManager,
    info_box: InfoBox,
    modal_stack: Vec<ModalBox>,
}

impl Scene {
    #[inline]
    pub fn new(state: &State) -> Self {
        let (w, h) = state.grid.size();
        Scene {
            grid_manager: GridManager::new((w / 2, h / 2)),
            info_box: InfoBox::new(&state.resources.font(FIRA_SANS_PATH, 16), &state),
            modal_stack: Vec::new(),
        }
    }
}

impl<'a> Behavior<State<'a>> for Scene {
    type Message = Message;

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        self.grid_manager.update(state, queue);
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.update(state, queue);
        };
    }

    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;

        trace!("Message: {:?}", message);
        if let ApplyOneModal = message {
            state.apply_one_modal(&mut self.modal_stack);
            return;
        }
        if state.will_pop_modals > 0 {
            return;
        }
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.handle(state, message, queue);
            return;
        }

        let manager = &mut self.grid_manager;
        match message {
            // Input
            Confirm => manager.confirm(state, queue),
            Cancel => manager.cancel(state, queue),
            RightReleasedAt(_, _) |
            CancelReleased => manager.cancel_release(),
            MoveCursorUp => manager.move_cursor_relative((0, 1), state),
            MoveCursorDown => manager.move_cursor_relative((0, -1), state),
            MoveCursorLeft => manager.move_cursor_relative((-1, 0), state),
            MoveCursorRight => manager.move_cursor_relative((1, 0), state),

            // Modal messages
            AttackSelected(pos, target) => {
                // manager.cursor.pos = target;
                manager.select_target(pos, target, state, queue);
            }
            WaitSelected => {
                // manager.cursor.pos = target;
                manager.hide_cursor();
            }
            CancelSelected(pos, target) => {
                state.grid.move_unit(target, pos);
                manager.move_cursor_to(pos, state);
                manager.hide_cursor();
                manager.select_unit(pos, state, queue);
            }
            TargetSelectorCanceled(origin, pos) => {
                manager.handle_unit_moved(origin, pos, state, queue);
            }

            // State changes
            UnitSpent(pos) => manager.unit_spent(pos, state),
            UnitMoved(from, to) => {
                let (_, unit) = state.active_unit.take().expect("no active unit after move");
                state.grid.add_unit(unit, to);
                manager.handle_unit_moved(from, to, state, queue);
            }
            TargetConfirmed(pos, target) => manager.target_confirmed(pos, target, state, queue),
            FinishTurn => {
                manager.deselect();
                for unit in state.grid.units_mut() {
                    unit.spent = false;
                }
                state.turn_info.end_turn();
                // TODO: Display a turn change animation here
            }

            MouseMovedTo(x, y) |
            LeftClickAt(x, y) |
            RightClickAt(x, y) => {
                manager.mouse_moved_to(x, y, state);
                match message {
                    MouseMovedTo(..) => {}
                    LeftClickAt(..) => {
                        manager.confirm(state, queue);
                    }
                    RightClickAt(..) => {
                        manager.cancel(state, queue);
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

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        self.grid_manager.render(state, renderer);
        self.info_box.render(state, renderer);
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.render(state, renderer);
        };
    }
}
