use glorious::{Behavior, Renderer};

use common::{GameObject, Message, State};

#[derive(Debug, Default)]
pub struct Scene<'a> {
    objects: Vec<GameObject<'a>>,
    modal_stack: Vec<GameObject<'a>>,
}

impl<'a> Scene<'a> {
    #[inline]
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            modal_stack: Vec::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, object: GameObject<'a>) {
        self.objects.push(object);
    }
}

impl<'a> Behavior<State<'a>> for Scene<'a> {
    type Message = Message;

    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        for object in &mut self.objects {
            object.initialize(state, queue);
        }
    }

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State<'a>, queue: &mut Vec<Message>) {
        for object in &mut self.objects {
            object.update(state, queue);
        }
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

    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        for object in &mut self.objects {
            object.render(state, renderer);
        }
        if let Some(modal) = self.modal_stack.last_mut() {
            modal.render(state, renderer);
        };
    }
}
