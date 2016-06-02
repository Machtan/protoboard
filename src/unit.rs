use std::rc::Rc;
use std::fmt::{self, Debug};
use sdl2::render::Texture;
use faction::Faction;

#[derive(Debug, Clone)]
pub struct Unit {
    pub health: u32,
    pub faction: Faction,
    pub spent: bool,
    unit_type: UnitType,
}

impl Unit {
    /// Attacks this unit and returns whether it gets destroyed.
    pub fn receive_attack(&mut self, attacker: &Unit) -> bool {
        if attacker.unit_type.damage >= self.health {
            self.health = 0;
            true
        } else {
            self.health -= attacker.unit_type.damage;
            false
        }
    }

    /// Returns the tiles in the attack range of this unit.
    pub fn tiles_in_attack_range(&self, pos: (u32, u32), grid_size: (u32, u32)) -> Vec<(u32, u32)> {
        self.unit_type.attack.tiles_in_range(pos, grid_size)
    }

    pub fn texture(&self) -> Rc<Texture> {
        self.unit_type.texture.clone()
    }
}

#[derive(Clone)]
pub struct UnitType {
    pub health: u32,
    pub attack: AttackType,
    pub damage: u32,
    pub texture: Rc<Texture>,
}

impl UnitType {
    #[inline]
    pub fn new(texture: Rc<Texture>,
               health: u32,
               attack_type: AttackType,
               damage: u32)
               -> UnitType {
        UnitType {
            health: health,
            attack: attack_type,
            damage: damage,
            texture: texture,
        }
    }

    /// Instantiates a unit of this type in the given faction.
    /// If not health is given, the unit starts with full health.
    pub fn instantiate(&self, faction: Faction, health: Option<u32>) -> Unit {
        Unit {
            unit_type: self.clone(),
            health: health.unwrap_or(self.health),
            faction: faction,
            spent: false,
        }
    }
}

impl Debug for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Unit")
            .field("health", &self.health)
            .field("attack", &self.attack)
            .field("damage", &self.damage)
            .field("texture", &(..))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum AttackType {
    Melee,
    Ranged {
        min: u32,
        max: u32,
    },
    Spear {
        range: u32,
    },
}

impl AttackType {
    pub fn tiles_in_range(&self, pos: (u32, u32), grid_size: (u32, u32)) -> Vec<(u32, u32)> {
        let mut tiles = Vec::new();
        let (col, row) = pos;
        let (n_cols, n_rows) = grid_size;
        let max_col = n_cols - 1;
        let max_row = n_rows - 1;
        match *self {
            _ => {
                // Melee
                if col > 0 {
                    tiles.push((col - 1, row));
                }
                if col < max_col {
                    tiles.push((col + 1, row));
                }
                if row > 0 {
                    tiles.push((col, row - 1));
                }
                if row < max_row {
                    tiles.push((col, row + 1));
                }
            }
        }
        tiles
    }
}
