use std::fmt::{self, Debug};
use std::rc::Rc;

use sdl2::render::Texture;

use faction::Faction;
use grid::Terrain;

#[derive(Clone, Debug)]
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

    #[inline]
    pub fn unit_type(&self) -> &UnitType {
        &self.unit_type
    }

    /// Returns the tiles in the attack range of this unit.
    pub fn tiles_in_attack_range(&self, pos: (u32, u32), grid_size: (u32, u32)) -> TilesInRange {
        self.unit_type.attack.tiles_in_range(pos, grid_size)
    }

    pub fn texture(&self) -> Rc<Texture> {
        self.unit_type.texture.clone()
    }

    pub fn is_ranged(&self) -> bool {
        self.unit_type.attack.is_ranged()
    }

    pub fn terrain_cost(&self, terrain: &Terrain) -> u32 {
        match *terrain {
            Terrain::Grass => 1,
            Terrain::Mountain => 4,
            Terrain::Woods => 2,
        }
    }
}

#[derive(Clone)]
pub struct UnitType {
    pub health: u32,
    pub attack: AttackType,
    pub damage: u32,
    pub movement: u32,
    pub texture: Rc<Texture>,
}

impl UnitType {
    /// Creates a unit of this type in the given faction.
    /// If not health is given, the unit starts with full health.
    pub fn create(&self, faction: Faction, health: Option<u32>) -> Unit {
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
        f.debug_struct("UnitType")
            .field("health", &self.health)
            .field("attack", &self.attack)
            .field("damage", &self.damage)
            .field("movement", &self.movement)
            .field("texture", &(..))
            .finish()
    }
}

#[derive(Clone)]
pub struct TilesInRange {
    pos: (u32, u32),
    cur: (i32, i32),
    size: (u32, u32),
    min: u32,
}

impl Iterator for TilesInRange {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let (x, y) = self.pos;
        let (w, h) = self.size;
        while self.cur != (0, self.min as i32 - 1) {
            let (dx, dy) = self.cur;

            let tx = x as i32 + dx;
            let ty = y as i32 + dy;

            self.cur = match (dx.signum(), dy.signum()) {
                (0, 1) | (1, 1) => (dx + 1, dy - 1), // N-E
                (1, 0) | (1, -1) => (dx - 1, dy - 1), // E-S
                (0, -1) | (-1, -1) => (dx - 1, dy + 1), // S-W
                (-1, 0) | (-1, 1) => {
                    // W-N
                    if dx == -1 {
                        (dx + 1, dy)
                    } else {
                        (dx + 1, dy + 1)
                    }
                }
                _ => unreachable!(),
            };

            if 0 <= tx && tx < w as i32 && 0 <= ty && ty < h as i32 {
                return Some((tx as u32, ty as u32));
            }
        }
        None
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
    pub fn tiles_in_range(&self, pos: (u32, u32), size: (u32, u32)) -> TilesInRange {
        let (min, max) = match *self {
            AttackType::Melee => (1, 1),
            AttackType::Ranged { min, max } => (min, max),
            AttackType::Spear { range } => (1, range),
        };
        TilesInRange {
            pos: pos,
            cur: (0, max as i32),
            size: size,
            min: min,
        }
    }

    pub fn is_ranged(&self) -> bool {
        use self::AttackType::*;
        match *self {
            Ranged { .. } => true,
            Melee | Spear { .. } => false,
        }
    }
}
