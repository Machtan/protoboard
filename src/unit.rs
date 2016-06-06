use std::fmt::{self, Debug};
use std::rc::Rc;

use sdl2::render::Texture;

use faction::Faction;
use terrain::Terrain;

#[derive(Clone, Debug)]
pub struct Unit {
    pub health: u32,
    pub faction: Faction,
    pub spent: bool,
    unit_type: UnitType,
}

impl Unit {
    // TODO: Optimally we should not use floats here, but rather get a
    // better idea of the units for the quantities.

    pub fn defense_bonus(&self, terrain: &Terrain) -> f64 {
        match *terrain {
            Terrain::Grass => 0.1,
            Terrain::Woods => 0.3,
            Terrain::Mountains => 0.5,
        }
    }

    pub fn attack_damage(&self, other: &Unit, terrain: &Terrain) -> f64 {
        let def = other.defense_bonus(terrain);
        let atk = self.unit_type.damage as f64;
        let atk_hp = self.health as f64 / self.unit_type.health as f64;
        let def_hp = other.health as f64 / other.unit_type.health as f64;
        atk * atk_hp * (1.0 - def * def_hp)
    }

    /// Attacks this unit and returns whether it gets destroyed.
    pub fn receive_attack(&mut self, terrain: &Terrain, attacker: &Unit) -> bool {
        let damage = attacker.attack_damage(self, terrain);
        assert!(damage >= 0.0, "damage calculation should never be negative");
        // TODO: Maybe truncate instead of rounding?
        self.health = self.health.saturating_sub(damage.round() as u32);
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
    pub fn can_spear_through(&self, _other: &Unit) -> bool {
        false
    }

    #[inline]
    pub fn can_move_through(&self, other: &Unit) -> bool {
        // TODO: Alliances? Neutrals?
        self.faction == other.faction
    }

    #[inline]
    pub fn can_attack(&self, other: &Unit) -> bool {
        // TODO: Alliances? Neutrals?
        self.faction != other.faction
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
