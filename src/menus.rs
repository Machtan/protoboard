use std::fmt::{self, Debug};
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use glorious::Behavior;
use common::{State, Message};

pub type Handler = Box<Fn(&mut State, Message, &mut Vec<Message>)>;

const PAD: u32 = 10;
pub struct ModalMenu {
    x: i32,
    y: i32,
    width: u32,
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
            width: 0,
            handler: handler,
            label_ids: label_ids,
        })
    }
}

impl Behavior for ModalMenu {
    type State = State;
    type Message = Message;
    
    /// Initializes the object when it is added to the game.
    fn initialize(&mut self, state: &mut State, _queue: &mut Vec<Message>, 
            renderer: &mut Renderer) {
        let mut max_width = 0;
        for (i, id) in self.label_ids.iter().enumerate() {
            state.resources.create_label(id, &self.font, &self.options[i],
                (0, 0, 0, 0), renderer).expect("Could not create label");
            let label = state.resources.label(id).unwrap();
            let w = label.rect.width();
            if w > max_width {
                max_width = w;
            }
        }
        self.width = max_width + 2 * PAD;
    }

    /// Updates the object each frame.
    fn update(&mut self, _state: &mut State, _queue: &mut Vec<Message>) {
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
        let font = state.resources.font(&self.font).unwrap();
        let height = PAD * 2 + font.recommended_line_spacing() as u32 * self.options.len() as u32;
        renderer.set_draw_color(Color::RGBA(200, 200, 255, 150));
        renderer.fill_rect(Rect::new(self.x, self.y, self.width, height)).unwrap();
        let mut y = self.y + PAD as i32;
        let x = self.x + PAD as i32;
        for id in &self.label_ids {
            let label = state.resources.label(id).unwrap();
            label.render(renderer, x, y, None);
            y += font.recommended_line_spacing();
        }
    }
}

impl Debug for ModalMenu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ModalMenu")
    }
}