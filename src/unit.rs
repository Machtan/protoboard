use std::fmt::{self, Debug};

use faction::Faction;
use info::Role;
use info::Terrain;

#[derive(Clone)]
pub struct Unit {
    pub health: u32,
    pub faction: Faction,
    pub spent: bool,
    pub role: Role,
}

impl Unit {
    pub fn new(role: Role, faction: Faction) -> Unit {
        Unit {
            health: 10,
            faction: faction,
            spent: false,
            role: role,
        }
    }

    // TODO: Optimally we should not use floats here, but rather get a
    // better idea of the units for the quantities.

    pub fn defense_bonus(&self, terrain: &Terrain) -> f64 {
        terrain.defense + self.role.defense.defense
    }

    pub fn attack_damage(&self, other: &Unit, terrain: &Terrain) -> f64 {
        let def = other.defense_bonus(terrain);
        let atk = self.role.attack.damage *
                  self.role.attack.modifiers.get(&other.role.defense.class).cloned().unwrap_or(1.0);
        let atk_hp = self.health as f64 / 10.0;
        let def_hp = other.health as f64 / 10.0;
        atk * atk_hp * (1.0 - def * def_hp)
    }

    pub fn retaliation_damage(&self, _damage_taken: f64, other: &Unit, terrain: &Terrain) -> f64 {
        self.attack_damage(other, terrain)
    }

    pub fn receive_damage(&mut self, damage: f64) -> bool {
        let damage = if damage < 0.0 {
            0.0
        } else {
            damage
        };
        self.health = self.health.saturating_sub(damage.round() as u32);
        self.health == 0
    }

    #[inline]
    pub fn terrain_cost(&self, terrain: &Terrain) -> u32 {
        *self.role
            .movement
            .class
            .costs
            .get(&terrain.name)
            .expect("missing terrain type in movement class")
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

impl Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Unit")
            .field("health", &self.health)
            .field("faction", &self.faction)
            .field("spent", &self.spent)
            .field("role", &self.role.name)
            .finish()
    }
}
