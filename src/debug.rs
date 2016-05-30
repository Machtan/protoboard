use glorious::Behavior;
use sdl2::render::Renderer;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use common::{GameObject, State, Message};
use unit::Unit;

pub struct DebugHelper;

impl Behavior for DebugHelper {
    type State = State;
    type Message = Message;
    
    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, _state: &mut Self::State, _new_messages: &mut Vec<Self::Message>) {
        // Do nothing by default
    }

    /// Updates the object each frame.
    fn update(&mut self, _state: &mut Self::State, _queue: &mut Vec<Self::Message>) {
        // Do nothing by default
    }

    /// Handles new messages since the last frame.
    fn handle(&mut self,
              state: &mut Self::State,
              messages: &[Self::Message],
              new_messages: &mut Vec<Self::Message>) {
        use common::Message::*;
        // Do nothing by default
        for message in messages {
            println!("[Debug] Message: {:?}", message);
            match *message {
                _ => {}
            }
        }
    }

    /// Renders the object.
    fn render(&self, state: &Self::State, renderer: &mut Renderer) {
        
    }
}