use common::{Message, State, GameObject};
use sdl2::render::Renderer;
use glorious::Behavior;

#[derive(Debug)]
pub struct Scene {
    objects: Vec<GameObject>,
    modal_stack: Vec<GameObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            modal_stack: Vec::new(),
        }
    }

    pub fn add(&mut self, object: GameObject) {
        self.objects.push(object);
    }
}

impl Default for Scene {
    fn default() -> Scene {
        Scene::new()
    }
}

impl Behavior for Scene {
    type State = State;
    type Message = Message;

    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        for object in &mut self.objects {
            object.initialize(state, queue);
        }
    }

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        state.apply_modal_stack(&mut self.modal_stack);

        for object in &mut self.objects {
            object.update(state, queue);
        }
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.update(state, queue);
        };
    }

    fn handle(&mut self, state: &mut State, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;

        trace!("Message: {:?}", message);

        match self.modal_stack.last_mut() {
            None => {
                for object in &mut self.objects {
                    object.handle(state, message.clone(), queue);
                }
            }
            Some(modal) => {
                modal.handle(state, message, queue);
            }
        }
    }

    fn render(&mut self, state: &Self::State, renderer: &mut Renderer) {
        for object in &mut self.objects {
            object.render(state, renderer);
        }
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.render(state, renderer);
        };
    }
}
