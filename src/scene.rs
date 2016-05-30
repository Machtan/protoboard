use common::{Message, State, GameObject};
use sdl2::render::Renderer;
use glorious::Behavior;

#[derive(Debug)]
pub enum SceneState {
    Normal,
    Modal(Vec<GameObject>),
}

#[derive(Debug)]
pub struct Scene {
    objects: Vec<GameObject>,
    state: SceneState,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            state: SceneState::Normal,
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

    fn handle(&mut self,
              state: &mut State,
              messages: &[Message],
              new_messages: &mut Vec<Message>) {
        use self::SceneState::*;
        match self.state {
            Normal => {
                for object in &mut self.objects {
                    object.handle(state, messages, new_messages);
                }
            }
            Modal(ref _modal_stack) => {
                for message in messages {
                    match *message {
                        _ => {}
                    }
                }
                // modal.handle(state, messages, new_messages);
            }
        }
    }

    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        for object in &self.objects {
            object.render(state, renderer);
        }
        if let SceneState::Modal(ref _modal_stack) = self.state {
            // modal.render(state, renderer);
        }
    }
}
