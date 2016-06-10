use info::Terrain;
use faction::Faction;

#[derive(Clone, Debug)]
pub struct Tile {
    pub terrain: Terrain,
    pub faction: Option<Faction>,
    pub capture: Option<(Faction, u32)>,
}

impl Tile {
    #[inline]
    pub fn can_be_captured(&self) -> bool {
        self.terrain.capture != 0
    }

    #[inline]
    pub fn capture(&mut self, faction: Faction, capture: u32) -> bool {
        let value = match self.capture {
            None => capture,
            Some((prev_faction, prev_value)) => {
                assert_eq!(prev_faction, faction);
                prev_value.saturating_add(capture)
            }
        };
        if value >= self.terrain.capture {
            self.faction = Some(faction);
            self.capture = None;
            true
        } else {
            self.capture = Some((faction, value));
            false
        }
    }
}
