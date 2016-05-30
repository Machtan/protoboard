use glorious::Behavior;
use sdl2::render::Renderer;

use common::{State, Message};

#[derive(Debug)]
pub struct DebugHelper;

impl Behavior for DebugHelper {
    type State = State;
    type Message = Message;

    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, _state: &mut State, _new_messages: &mut Vec<Message>) {
        // Do nothing.
    }

    /// Updates the object each frame.
    fn update(&mut self, _state: &mut State, _queue: &mut Vec<Message>) {
        // Do nothing.
    }

    /// Handles new messages since the last frame.
    fn handle(&mut self,
              _state: &mut State,
              message: Message,
              _queue: &mut Vec<Message>) {
        println!("[Debug] Message: {:?}", message);
    }

    /// Renders the object.
    fn render(&self, _state: &State, _renderer: &mut Renderer) {
        // Do nothing.
    }
}
