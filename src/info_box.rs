use std::cmp;
use std::collections::HashMap;

use glorious::{Behavior, Color, Label, Renderer};
use sdl2::rect::Rect;
use sdl2_ttf::Font;

use common::{Message, State};
use faction::Faction;

const BG_COLOR: Color = Color(0, 0, 0, 0x77);
const TEXT_COLOR: Color = Color(0xff, 0xff, 0xff, 0xff);
const POS: (i32, i32) = (400, 50);

#[derive(Debug)]
pub struct InfoBox {
    line_spacing: u32,
    faction_label: Label,
    actions_label: Label,
    faction_labels: HashMap<Faction, Label>,
    number_labels: Vec<Label>,
    max_num_width: u32,
}

impl InfoBox {
    pub fn new(font: &Font, state: &State) -> InfoBox {
        let (_, scale_y) = state.resources.device().scale();
        let line_spacing = font.recommended_line_spacing();
        let line_spacing = (line_spacing as f32 / scale_y) as u32;
        let faction_label = Label::new(font,
                                       "Current faction:   ",
                                       TEXT_COLOR,
                                       state.resources.device());
        let actions_label =
            Label::new(&font, "Actions left:", TEXT_COLOR, state.resources.device());
        let mut faction_labels = HashMap::new();
        for &faction in &state.turn_info.factions {
            let label = Label::new(font,
                                   &format!("{:?}", faction),
                                   TEXT_COLOR,
                                   state.resources.device());
            faction_labels.insert(faction, label);
        }
        let mut number_labels = Vec::new();
        let mut max_width = 0;
        for number in 0..state.turn_info.max_actions_left + 1 {
            let label = Label::new(font,
                                   &format!("{}", number),
                                   TEXT_COLOR,
                                   state.resources.device());
            let (width, _) = label.size();
            max_width = cmp::max(max_width, width);
            number_labels.push(label);
        }
        InfoBox {
            line_spacing: line_spacing,
            faction_label: faction_label,
            actions_label: actions_label,
            faction_labels: faction_labels,
            number_labels: number_labels,
            max_num_width: max_width,
        }
    }
}

impl<'a> Behavior<State<'a>> for InfoBox {
    type Message = Message;

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        // Render which faction's turn it is.
        // Render the amount of actions left somewhere.
        let (x, y) = POS;
        let (w, _) = self.faction_label.size();
        let right = x + w as i32;

        let rect = Rect::new(x - 5, y, 200, 50);
        renderer.set_draw_color(BG_COLOR);
        renderer.fill_rect(rect).unwrap();

        self.faction_label.render(renderer, x, y);
        self.faction_labels
            .get_mut(&state.turn_info.current_faction())
            .expect("Invalid current faction")
            .render(renderer, right, y);
        let second = y + self.line_spacing as i32;
        self.actions_label.render(renderer, x, second);
        self.number_labels
            .get_mut(state.turn_info.actions_left as usize)
            .expect("Invalid number of actions left")
            .render(renderer, right, second);
    }
}
