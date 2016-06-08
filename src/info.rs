use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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

#[derive(Clone, Debug)]
pub struct SpriteInfo {
    pub texture: String,
    pub area: Option<(u32, u32, u32, u32)>,
}

#[derive(Clone, Debug)]
pub struct TerrainInfo {
    pub name: String,
    pub defense: f64,
    pub sprite: Option<SpriteInfo>,
}

pub type Terrain = Rc<TerrainInfo>;

#[derive(Clone, Debug)]
pub struct AttackInfo {
    pub damage: f64,
    pub range: RangeKind,
    pub modifiers: HashMap<String, f64>,
}

#[derive(Clone, Debug)]
pub struct DefenseInfo {
    pub defense: f64,
    pub class: String,
}

#[derive(Clone, Debug)]
pub struct MovementClassInfo {
    pub name: String,
    pub costs: HashMap<String, u32>,
}

pub type MovementClass = Rc<MovementClassInfo>;

#[derive(Clone, Debug)]
pub struct MovementInfo {
    pub movement: u32,
    pub class: MovementClass,
}

#[derive(Clone, Debug)]
pub struct RoleInfo {
    pub name: String,
    pub attack: AttackInfo,
    pub defense: DefenseInfo,
    pub movement: MovementInfo,
    pub sprite: SpriteInfo,
}

pub type Role = Rc<RoleInfo>;

#[derive(Clone, Debug)]
pub struct GameInfo {
    pub movement_classes: HashMap<String, MovementClass>,
    pub roles: HashMap<String, Role>,
    pub terrain: HashMap<String, Terrain>,
    pub defense_classes: HashSet<String>,
}
