
use sdl2::render::Renderer;
use glorious::Behavior;
use common::{State, Message, GameObject};

pub type Handler = Box<Fn(&mut State, Message, &mut Vec<Message>)>;

pub struct ModalMenu {
    x: i32,
    y: i32,
    font: String,
    options: Vec<String>,
    label_ids: Vec<String>,
    selected: Option<usize>,
    handler: Handler,
}

impl ModalMenu {
    pub fn new(options: &[&str], selected: usize, x: i32, y: i32, font: &str, 
            handler: Handler) 
            -> Result<ModalMenu, String> {
        if options.is_empty() {
            return Err(String::from("No options given for the menu"));
        }
        if selected >= options.len() {
            return Err(format!("The selected option {} is not a valid index for the number of options; {}", 
                selected, options.len()));
        }
        let font = String::from(font);
        let mut label_ids = Vec::new();
        for option in options {
            let mut id = font.clone();
            id.push('/');
            id.push_str(option);
            label_ids.push(id);
        }
        Ok(ModalMenu {
            options: options.iter().map(|s| s.to_string()).collect(),
            selected: Some(selected),
            x: x,
            y: y,
            font: font,
            handler: handler,
            label_ids: label_ids,
        })
    }
}

impl Behavior for ModalMenu {
    type State = State;
    type Message = Message;
    
    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, state: &mut State, queue: &mut Vec<Message>, 
            renderer: &mut Renderer) {
        for (i, id) in self.label_ids.iter().enumerate() {
            state.resources.create_label(id, &self.font, &self.options[i],
                (0, 0, 0, 0), renderer);
        }
    }

    /// Updates the object each frame.
    fn update(&mut self, state: &mut State, queue: &mut Vec<Message>) {
    }

    /// Handles new messages since the last frame.
    fn handle(&mut self,
              state: &mut State,
              message: Message,
              queue: &mut Vec<Message>) {
        (self.handler)(state, message, queue);
    }

    /// Renders the object.
    fn render(&self, state: &State, renderer: &mut Renderer) {
        
    }
}