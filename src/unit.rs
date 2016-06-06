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
    pub fn receive_attack(&mut self, terrain: &Terrain, attacker: &Unit) -> bool {
        let terrain_bonus = match *terrain {
            Terrain::Grass => 0,
            Terrain::Woods => 2,
            Terrain::Mountains => 3,
        };
        let damage = attacker.unit_type.damage.saturating_sub(terrain_bonus);
        self.health = self.health.saturating_sub(damage);
        self.health == 0
    }

    #[inline]
    pub fn unit_type(&self) -> &UnitType {
        &self.unit_type
    }

    #[inline]
    pub fn texture(&self) -> Rc<Texture> {
        self.unit_type.texture.clone()
    }

    #[inline]
    pub fn terrain_cost(&self, terrain: &Terrain) -> u32 {
        match *terrain {
            Terrain::Grass => 1,
            Terrain::Mountains => 4,
            Terrain::Woods => 2,
        }
    }

    #[inline]
    pub fn can_spear_through(&self, other: &Unit) -> bool {
        // TODO: Alliances? Neutrals?
        self.faction == other.faction
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
