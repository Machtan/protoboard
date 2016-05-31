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

    /// Initializes the object when it is added to the game.
    fn initialize(&mut self,
                  state: &mut State,
                  queue: &mut Vec<Message>,
                  renderer: &mut Renderer) {
        for object in &mut self.objects {
            object.initialize(state, queue, renderer);
        }
    }

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State, queue: &mut Vec<Message>) {
        for object in &mut self.objects {
            object.update(state, queue);
        }
        if let SceneState::Modal(ref mut modal_stack) = self.state {
            let last = modal_stack.len() - 1;
            let ref mut modal = modal_stack[last];
            modal.update(state, queue);
        }
    }

    fn handle(&mut self, state: &mut State, message: Message, queue: &mut Vec<Message>) {
        use self::SceneState::*;
        use common::Message::*;

        trace!("Message: {:?}", message);

        let mut break_modal = false;
        match self.state {
            Normal => {
                if let PushModal(modal_obj) = message {
                    info!("Modal ENTER");
                    self.state = Modal(vec![modal_obj]);
                    return;
                }
                for object in &mut self.objects {
                    if let Some(m) = message.try_clone() {
                        object.handle(state, m, queue);
                    }
                }
            }
            Modal(ref mut modal_stack) => {
                match message {
                    PushModal(modal_obj) => {
                        modal_stack.push(modal_obj);
                    }
                    PopModal => {
                        if modal_stack.len() == 1 {
                            break_modal = true;
                        } else {
                            modal_stack.pop();
                        }
                    }
                    BreakModal => {
                        break_modal = true;
                    }
                    other_message => {
                        let last = modal_stack.len() - 1;
                        let ref mut modal = modal_stack[last];
                        modal.handle(state, other_message, queue);
                    }
                }
            }
        }
        if break_modal {
            info!("Modal EXIT");
            self.state = Normal;
        }
    }

    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        for object in &self.objects {
            object.render(state, renderer);
        }
        if let SceneState::Modal(ref modal_stack) = self.state {
            let last = modal_stack.len() - 1;
            let ref modal = modal_stack[last];
            modal.render(state, renderer);
        }
    }
}
