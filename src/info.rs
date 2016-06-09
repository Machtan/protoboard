use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use spec::*;

#[derive(Debug, Clone)]
pub enum RangeKind {
    Melee,
    Ranged {
        min: u32,
        max: u32,
    },
    Spear {
        range: u32,
    },
}

impl RangeKind {
    fn from_spec(spec: RangeSpec) -> Result<RangeKind, String> {
        Ok(match &spec.kind[..] {
            "melee" => RangeKind::Melee,
            "ranged" => {
                RangeKind::Ranged {
                    min: spec.min.ok_or_else(|| "missing field 'min' for ranged range".to_owned())?,
                    max: spec.max.ok_or_else(|| "missing field 'max' for ranged range".to_owned())?,
                }
            }
            "spear" => {
                RangeKind::Spear {
                    range: spec.range
                        .ok_or_else(|| "missing field 'range' for spear range".to_owned())?,
                }
            }
            kind => return Err(format!("unrecognized range kind {:?}", kind)),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SpriteInfo {
    pub texture: String,
    pub area: Option<(u32, u32, u32, u32)>,
}

impl SpriteInfo {
    #[inline]
    fn from_spec(spec: SpriteSpec) -> Result<SpriteInfo, String> {
        Ok(SpriteInfo {
            texture: spec.texture,
            area: spec.area,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TerrainInfo {
    pub name: String,
    pub defense: f64,
    pub sprite: Option<SpriteInfo>,
}

impl TerrainInfo {
    #[inline]
    fn from_spec(spec: TerrainSpec, name: String) -> Result<TerrainInfo, String> {
        let sprite = match spec.sprite {
            Some(spec) => Some(SpriteInfo::from_spec(spec)?),
            None => None,
        };
        Ok(TerrainInfo {
            name: name,
            defense: spec.defense,
            sprite: sprite,
        })
    }
}

pub type Terrain = Rc<TerrainInfo>;

#[derive(Clone, Debug)]
pub struct AttackInfo {
    pub damage: f64,
    pub range: RangeKind,
    pub modifiers: HashMap<String, f64>,
}

impl AttackInfo {
    #[inline]
    fn from_spec(spec: AttackSpec) -> Result<AttackInfo, String> {
        Ok(AttackInfo {
            damage: spec.damage,
            range: RangeKind::from_spec(spec.range)?,
            modifiers: spec.modifiers,
        })
    }
}

#[derive(Clone, Debug)]
pub struct DefenseInfo {
    pub defense: f64,
    pub class: String,
}

impl DefenseInfo {
    #[inline]
    fn from_spec(spec: DefenseSpec) -> Result<DefenseInfo, String> {
        Ok(DefenseInfo {
            defense: spec.defense,
            class: spec.class,
        })
    }
}

#[derive(Clone, Debug)]
pub struct MovementClassInfo {
    pub name: String,
    pub costs: HashMap<String, u32>,
}

impl MovementClassInfo {
    #[inline]
    fn from_spec(spec: MovementClassSpec, name: String) -> Result<MovementClassInfo, String> {
        Ok(MovementClassInfo {
            name: name,
            costs: spec,
        })
    }
}

pub type MovementClass = Rc<MovementClassInfo>;

#[derive(Clone, Debug)]
pub struct MovementInfo {
    pub movement: u32,
    pub class: MovementClass,
}

impl MovementInfo {
    #[inline]
    fn from_spec<F>(spec: MovementSpec, mut to_movement_class: F) -> Result<MovementInfo, String>
        where F: FnMut(&str) -> Option<MovementClass>,
    {
        Ok(MovementInfo {
            movement: spec.movement,
            class: to_movement_class(&spec.class).ok_or_else(|| format!("unrecognized movement class {:?}", spec.class))?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RoleInfo {
    pub name: String,
    pub attack: AttackInfo,
    pub defense: DefenseInfo,
    pub movement: MovementInfo,
    pub sprite: SpriteInfo,
}

impl RoleInfo {
    #[inline]
    fn from_spec<F>(spec: RoleSpec, name: String, to_movement_class: F) -> Result<RoleInfo, String>
        where F: FnMut(&str) -> Option<MovementClass>
    {
        Ok(RoleInfo {
            name: name,
            attack: AttackInfo::from_spec(spec.attack)?,
            defense: DefenseInfo::from_spec(spec.defense)?,
            movement: MovementInfo::from_spec(spec.movement, to_movement_class)?,
            sprite: SpriteInfo::from_spec(spec.sprite)?,
        })
    }
}

pub type Role = Rc<RoleInfo>;

#[derive(Clone, Debug)]
pub struct GameInfo {
    pub movement_classes: HashMap<String, MovementClass>,
    pub roles: HashMap<String, Role>,
    pub terrain: HashMap<String, Terrain>,
    pub defense_classes: HashSet<String>,
}

impl GameInfo {
    pub fn from_spec(spec: Spec) -> Result<GameInfo, String> {
        let terrain = spec.terrain
            .into_iter()
            .map(|(name, spec)| {
                let info = TerrainInfo::from_spec(spec, name.clone())?;
                Ok((name, Terrain::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        let movement_classes = spec.movement_classes
            .into_iter()
            .map(|(name, spec)| {
                let info = MovementClassInfo::from_spec(spec, name.clone())?;

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

                Ok((name, MovementClass::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        let roles = spec.roles
            .into_iter()
            .map(|(name, spec)| {
                let info = RoleInfo::from_spec(spec, name.clone(), |m| movement_classes.get(m).cloned())?;
                Ok((name, Role::new(info)))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;

        Ok(GameInfo {
            movement_classes: movement_classes,
            roles: roles,
            terrain: terrain,
            defense_classes: spec.defense_classes,
        })
    }
}
