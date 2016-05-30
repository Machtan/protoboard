use common::{Message, State, GameObject};
use sdl2::render::Renderer;

use glorious::Behavior;

pub struct Scene {
    objects: Vec<GameObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { objects: Vec::new() }
    }

    pub fn add(&mut self, object: GameObject) {
        self.objects.push(object);
    }
}

impl Behavior for Scene {
    type State = State;
    type Message = Message;

    fn handle(&mut self,
              state: &mut Self::State,
              messages: &[Self::Message],
              new_messages: &mut Vec<Self::Message>) {
        use common::Message::*;
        for message in messages.iter() {
            match message {
                _ => {}
            }
        }
        for object in &mut self.objects {
            object.handle(state, messages, new_messages);
        }
    }

    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        for object in &self.objects {
            object.render(state, renderer);
        }
    }
}
