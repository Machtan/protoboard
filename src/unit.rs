use std::rc::Rc;
use std::fmt::{self, Debug};
use sdl2::render::Texture;

#[derive(Clone)]
pub struct Unit {
    pub max_health: u32,
    pub health: u32,
    pub attack: AttackType,
    pub damage: u32,
    pub spent: bool,
    pub texture: Rc<Texture>,
}

impl Unit {
    #[inline]
    pub fn new(texture: Rc<Texture>, max_health: u32, attack_type: AttackType, damage: u32) -> Unit {
        Unit {
            max_health: max_health,
            health: max_health,
            attack: attack_type,
            damage: damage,
            spent: false,
            texture: texture,
        }
    }
    
    /// Returns whether this unit is destroyed.
    pub fn on_attack(&mut self, attacker: &Unit) -> bool {
        if attacker.damage >= self.health {
            self.health = 0;
            true
        } else {
            self.health -= attacker.damage;
            false
        }
    }
}

impl Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Unit")
            .field("attack", &self.attack)
            .field("spent", &self.spent)
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
