use glorious::{Behavior, Renderer};

use common::{GameObject, Message, State};
use grid_manager::GridManager;
use info_box::InfoBox;
use resources::FIRA_SANS_PATH;

#[derive(Debug)]
pub struct Scene {
    grid_manager: GridManager,
    info_box: InfoBox,
    modal_stack: Vec<GameObject>,
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
        trace!("Message: {:?}", message);
        if let Message::ApplyOneModal = message {
            state.apply_one_modal(&mut self.modal_stack);
            return;
        }
        if state.will_pop_modals > 0 {
            return;
        }
        match self.modal_stack.last_mut() {
            None => {
                self.grid_manager.handle(state, message, queue);
            }
            Some(modal) => {
                modal.handle(state, message, queue);
            }
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
