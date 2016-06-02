use std::rc::Rc;
use std::borrow::Cow;
use std::fmt::{self, Debug};
use sdl2::render::Texture;
use faction::Faction;

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

    /// Returns the tiles in the attack range of this unit.
    pub fn tiles_in_attack_range(&self, pos: (u32, u32), grid_size: (u32, u32)) -> TilesInRange {
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
            .field("texture", &(..))
            .finish()
    }
}

const DELTAS_MELEE: &'static [(i32, i32)] = &[(-1, 0), (0, -1), (1, 0), (0, 1)];

pub struct TilesInRange {
    deltas: Vec<(i32, i32)>,
    index: usize,
    pos: (u32, u32),
    size: (u32, u32),
}

fn ranged_deltas(min: u32, max: u32) -> Vec<(i32, i32)> {
    let min = min as i32;
    let max = max as i32;
    let mut deltas = Vec::new();
    for i in 0..max {
        let col = max - i;
        if col > min {
            deltas.push((-col, 0));
            deltas.push((col, 0));
            deltas.push((0, col));
            deltas.push((0, -col));
        }
        let row_end = max - col;
        let delta = max - min;
        let row_start = if row_end < delta {
            1
        } else {
            row_end - delta
        };
        for row in row_start..row_end {
            deltas.push((col, row));
            deltas.push((col, -row));
            deltas.push((-col, row));
            deltas.push((-col, -row));
        }
    }
    deltas
}

impl Iterator for TilesInRange {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let (x, y) = self.pos;
        let (w, h) = self.size;
        loop {
            if self.index >= self.deltas.len() {
                return None;
            }
            let (dx, dy) = self.deltas[self.index];

            let tx = x as i32 + dx;
            let ty = y as i32 + dy;

            if 0 <= tx && tx < w as i32 && 0 <= ty && ty < h as i32 {
                self.index += 1;
                return Some((tx as u32, ty as u32));
            } else {
                self.index += 1;
            }
        }
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
        let deltas = match *self {
            AttackType::Melee => DELTAS_MELEE.iter().map(|&d| d).collect(),
            AttackType::Ranged { min, max } => ranged_deltas(min, max),
            _ => Vec::new(),
        };
        TilesInRange {
            deltas: deltas,
            index: 0,
            pos: pos,
            size: size,
        }
    }
}
