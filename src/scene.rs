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
            info_box: InfoBox::new(&state.resources.font(FIRA_SANS_PATH, 16), state),
            modal_stack: Vec::new(),
        }
    }
}

impl<'a> Behavior<State<'a>> for Scene {
    type Message = Message;

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        let mut defeated = Vec::new();
        for &faction in state.turn_info.factions() {
            if state.grid.units().all(|u| u.faction != faction) {
                defeated.push(faction);
            }
        }
        if !defeated.is_empty() {
            for faction in defeated {
                info!("Faction defeated: {:?}", faction);
                state.turn_info.remove_faction(faction);
            }
            match state.turn_info.factions().split_last() {
                None => info!("No contest; everybody loses."),
                Some((&faction, rest)) => {
                    if rest.iter().all(|&f| f == faction) {
                        info!("We have a winner! {:?}!", faction);
                    }
                }
            }
        }

        self.grid_manager.update(state);
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
            Confirm => {
                if let Some(modal) = manager.confirm(state) {
                    // TODO
                    state.push_modal(modal, queue);
                }
            }
            Cancel => manager.cancel(state),
            RightReleasedAt(_, _) |
            CancelReleased => manager.cancel_release(),
            MoveCursorUp => manager.move_cursor_relative((0, 1), state),
            MoveCursorDown => manager.move_cursor_relative((0, -1), state),
            MoveCursorLeft => manager.move_cursor_relative((-1, 0), state),
            MoveCursorRight => manager.move_cursor_relative((1, 0), state),

            // Modal messages
            AttackSelected(pos, target) => {
                // manager.cursor.pos = target;
                let modal = manager.select_target(pos, target, state);
                // TODO
                state.push_modal(modal, queue);
            }
            CaptureSelected(pos) => manager.capture_at(pos, state),
            WaitSelected => {
                // manager.cursor.pos = target;
                manager.hide_cursor();
            }
            CancelSelected(pos, target) => {
                state.grid.move_unit(target, pos);
                manager.move_cursor_to(pos, state);
                manager.hide_cursor();
                manager.select_unit(pos, state);
            }
            TargetSelectorCanceled(origin, pos) => {
                let modal = manager.handle_unit_moved(origin, pos, state);
                // TODO
                state.push_modal(modal, queue);
            }

            // State changes
            UnitSpent(pos) => manager.unit_spent(pos, state),
            UnitMoved(from, to) => {
                let (_, unit) = state.active_unit.take().expect("no active unit after move");
                state.grid.add_unit(unit, to);
                let modal = manager.handle_unit_moved(from, to, state);
                // TODO
                state.push_modal(modal, queue);
            }
            TargetConfirmed(pos, target) => manager.target_confirmed(pos, target, state),
            FinishTurn => {
                manager.deselect();
                for unit in state.grid.units_mut() {
                    unit.spent = false;
                }
                state.turn_info.end_turn();
                // TODO: Display a turn change animation here
            }

            MouseMovedTo(x, y) => manager.mouse_moved_to(x, y, state),
            LeftClickAt(x, y) => {
                manager.mouse_moved_to(x, y, state);
                if let Some(modal) = manager.confirm(state) {
                    // TODO
                    state.push_modal(modal, queue);
                }
            }
            RightClickAt(x, y) => {
                manager.mouse_moved_to(x, y, state);
                manager.cancel(state);
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
