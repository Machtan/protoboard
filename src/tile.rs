use info::Terrain;
use faction::Faction;

#[derive(Clone, Debug)]
pub struct Tile {
    pub terrain: Terrain,
    pub faction: Option<Faction>,
    pub capture: Option<(Faction, u32)>,
    pub capture_health: u32,
}

impl Tile {
    #[inline]
    pub fn can_be_captured(&self) -> bool {
        self.capture_health != 0
    }
}
