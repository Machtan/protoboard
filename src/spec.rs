use std::collections::{HashMap, HashSet};

use info::*;

#[derive(Deserialize)]
struct RangeSpec {
    kind: String,
    min: Option<u32>,
    max: Option<u32>,
    range: Option<u32>,
}

impl RangeSpec {
    fn to_info(&self) -> Result<RangeKind, String> {
        Ok(match &self.kind[..] {
            "melee" => RangeKind::Melee,
            "ranged" => {
                RangeKind::Ranged {
                    min: self.min.ok_or_else(|| format!("missing field 'min' for ranged range"))?,
                    max: self.max.ok_or_else(|| format!("missing field 'max' for ranged range"))?,
                }
            }
            "spear" => {
                RangeKind::Spear {
                    range: self.range
                        .ok_or_else(|| format!("missing field 'range' for spear range"))?,
                }
            }
            kind => return Err(format!("unrecognized range kind {:?}", kind)),
        })
    }
}

#[derive(Deserialize)]
struct SpriteSpec {
    texture: String,
    area: Option<(u32, u32, u32, u32)>,
}

impl SpriteSpec {
    fn to_info(&self) -> Result<SpriteInfo, String> {
        Ok(SpriteInfo {
            texture: self.texture.clone(),
            area: self.area,
        })
    }
}

#[derive(Deserialize)]
struct TerrainSpec {
    defense: f64,
    sprite: Option<SpriteSpec>,
}

impl TerrainSpec {
    fn to_info(&self, name: String) -> Result<TerrainInfo, String> {
        let sprite = match self.sprite {
            Some(ref spec) => Some(spec.to_info()?),
            None => None,
        };
        Ok(TerrainInfo {
            name: name,
            defense: self.defense,
            sprite: sprite,
        })
    }
}

#[derive(Deserialize)]
struct AttackSpec {
    damage: f64,
    range: RangeSpec,
    modifiers: HashMap<String, f64>,
}

impl AttackSpec {
    fn to_info(&self) -> Result<AttackInfo, String> {
        Ok(AttackInfo {
            damage: self.damage,
            range: self.range.to_info()?,
            modifiers: self.modifiers.clone(),
        })
    }
}

#[derive(Deserialize)]
struct DefenseSpec {
    defense: f64,
    class: String,
}

impl DefenseSpec {
    fn to_info(&self) -> Result<DefenseInfo, String> {
        Ok(DefenseInfo {
            defense: self.defense,
            class: self.class.clone(),
        })
    }
}

#[derive(Deserialize)]
struct MovementSpec {
    movement: u32,
    class: String,
}

impl MovementSpec {
    fn to_info<F>(&self, mut to_movement_class: F) -> Result<MovementInfo, String>
        where F: FnMut(&str) -> Option<MovementClass>
    {
        Ok(MovementInfo {
            movement: self.movement,
            class: to_movement_class(&self.class)
                .ok_or_else(|| format!("unrecognized movement class: {:?}", self.class))?,
        })
    }
}

#[derive(Deserialize)]
struct RoleSpec {
    attack: AttackSpec,
    defense: DefenseSpec,
    movement: MovementSpec,
    sprite: SpriteSpec,
}

impl RoleSpec {
    fn to_info<F>(&self, name: String, to_movement_class: F) -> Result<RoleInfo, String>
        where F: FnMut(&str) -> Option<MovementClass>
    {
        Ok(RoleInfo {
            name: name,
            attack: self.attack.to_info()?,
            defense: self.defense.to_info()?,
            movement: self.movement.to_info(to_movement_class)?,
            sprite: self.sprite.to_info()?,
        })
    }
}

#[derive(Deserialize)]
pub struct Spec {
    movement_classes: HashMap<String, HashMap<String, u32>>,
    roles: HashMap<String, RoleSpec>,
    terrain: HashMap<String, TerrainSpec>,
    defense_classes: HashSet<String>,
}

impl Spec {
    pub fn to_info(&self) -> Result<GameInfo, String> {
        let terrain = self.terrain
            .iter()
            .map(|(name, spec)| {
                let info = spec.to_info(name.clone())?;
                Ok((name.clone(), Terrain::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        let movement_classes = self.movement_classes
            .iter()
            .map(|(name, spec)| {
                let info = MovementClassInfo {
                    name: name.clone(),
                    costs: spec.clone(),
                };

                for tname in info.costs.keys() {
                    if !terrain.contains_key(&tname[..]) {
                        return Err(format!("unrecognized terrain {:?} for movement class {:?}",
                                           tname,
                                           name));
                    }
                }

                for tname in terrain.keys() {
                    if !info.costs.contains_key(&tname[..]) {
                        return Err(format!("movement class {:?} is missing terrain {:?}",
                                           name,
                                           tname));
                    }
                }

                Ok((name.clone(), MovementClass::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        let defense_classes = self.defense_classes.clone();

        let roles = self.roles
            .iter()
            .map(|(name, spec)| {
                let info = spec.to_info(name.clone(), |m| movement_classes.get(m).cloned())?;
                Ok((name.clone(), Role::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        Ok(GameInfo {
            movement_classes: movement_classes,
            roles: roles,
            terrain: terrain,
            defense_classes: defense_classes,
        })
    }
}
