
use glorious::{Behavior, Renderer};
use common::{Message, State};
use faction::Faction;

#[derive(Debug)]
pub struct TurnManager {
    action_limit: u32,
    factions: Vec<Faction>,
}

impl TurnManager {
    pub fn new(action_limit: u32, factions: Vec<Faction>) -> TurnManager {
        TurnManager {
            action_limit: action_limit,
            factions: factions,
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
    fn render(&mut self, _state: &State<'a>, _renderer: &mut Renderer) {
        // Render which faction's turn it is.
        // Render the amount of actions left somewhere.
    }
}
