use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;

use glorious::{Behavior, Label, Renderer};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::Font;

use common::{Message, State};
use faction::Faction;

const BG_COLOR: (u8, u8, u8, u8) = (0, 0, 0, 0x77);
const TEXT_COLOR: (u8, u8, u8, u8) = (0xff, 0xff, 0xff, 0xff);
const POS: (i32, i32) = (400, 50);

#[derive(Debug)]
pub struct TurnManager {
    action_limit: u32,
    factions: Vec<Faction>,
    line_spacing: u32,
    faction_label: Label,
    actions_label: Label,
    faction_labels: HashMap<Faction, Label>,
    number_labels: Vec<Label>,
    max_num_width: u32,
}

impl TurnManager {
    pub fn new(action_limit: u32,
               factions: Vec<Faction>,
               font: Rc<Font>,
               state: &State)
               -> TurnManager {
        let (_, scale_y) = state.resources.renderer().scale();
        let line_spacing = font.recommended_line_spacing();
        let line_spacing = (line_spacing as f32 / scale_y) as u32;
        let faction_label = Label::new(font.clone(),
                                       "Current faction:   ",
                                       TEXT_COLOR,
                                       state.resources.renderer());
        let actions_label = Label::new(font.clone(),
                                       "Actions left:",
                                       TEXT_COLOR,
                                       state.resources.renderer());
        let mut faction_labels = HashMap::new();
        for &faction in &factions {
            let label = Label::new(font.clone(),
                                   format!("{:?}", faction),
                                   TEXT_COLOR,
                                   state.resources.renderer());
            faction_labels.insert(faction, label);
        }
        let mut number_labels = Vec::new();
        let mut max_width = 0;
        for number in 0..action_limit + 1 {
            let label = Label::new(font.clone(),
                                   format!("{}", number),
                                   TEXT_COLOR,
                                   state.resources.renderer());
            max_width = cmp::max(max_width, label.width());
            number_labels.push(label);
        }
        TurnManager {
            action_limit: action_limit,
            factions: factions,
            line_spacing: line_spacing,
            faction_label: faction_label,
            actions_label: actions_label,
            faction_labels: faction_labels,
            number_labels: number_labels,
            max_num_width: max_width,
        }
    }

    fn find_faction(&self, faction: Faction) -> Option<usize> {
        self.factions.iter().enumerate().find(|&(_, &f)| f == faction).map(|(i, _)| i)
    }
}

impl<'a> Behavior<State<'a>> for TurnManager {
    type Message = Message;

    /// Handles new messages since the last frame.
    fn handle(&mut self, state: &mut State<'a>, message: Message, queue: &mut Vec<Message>) {
        use common::Message::*;
        match message {
            FinishTurn => {
                let faction = state.current_turn;
                let i = self.find_faction(faction).expect("Invalid current faction");
                let next = (i + 1) % self.factions.len();
                state.current_turn = self.factions[next];
                state.actions_left = self.action_limit;
                // TODO: Display a turn change animation here
            }
            UnitSpent(_) => {
                assert!(state.actions_left != 0,
                        "A unit was spent with no actions left");
                state.actions_left -= 1;
            }
            FactionDefeated(faction) => {
                let i = self.find_faction(faction).expect("Invalid faction defeated");
                info!("Faction defeated! {:?}", faction);
                self.factions.remove(i);
                if self.factions.len() == 1 {
                    queue.push(FactionWins(self.factions[0]));
                }
            }
            _ => {}
        }
    }

    /// Renders the object.
    fn render(&mut self, state: &State<'a>, renderer: &mut Renderer) {
        // Render which faction's turn it is.
        // Render the amount of actions left somewhere.
        let (x, y) = POS;

        let right = x + self.faction_label.width() as i32;

        let rect = Rect::new(x - 5, y, 200, 50);
        let (r, g, b, a) = BG_COLOR;
        renderer.set_draw_color(Color::RGBA(r, g, b, a));
        renderer.fill_rect(rect).unwrap();

        self.faction_label.render(renderer, x, y);
        self.faction_labels
            .get_mut(&state.current_turn)
            .expect("Invalid current faction")
            .render(renderer, right, y);
        let second = y + self.line_spacing as i32;
        self.actions_label.render(renderer, x, second);
        self.number_labels
            .get_mut(state.actions_left as usize)
            .expect("Invalid number of actions left")
            .render(renderer, right, second);
    }
}
