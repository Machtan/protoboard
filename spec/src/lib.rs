#![feature(custom_derive)]
#![feature(plugin)]

#![plugin(serde_macros)]

use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Deserialize)]
pub struct RangeSpec {
    pub kind: String,
    pub min: Option<u32>,
    pub max: Option<u32>,
    pub range: Option<u32>,
}

#[derive(Deserialize)]
pub struct SpriteSpec {
    pub texture: String,
    pub area: Option<(u32, u32, u32, u32)>,
}

#[derive(Deserialize)]
pub struct TerrainSpec {
    pub defense: f64,
    pub sprite: Option<SpriteSpec>,
}

#[derive(Deserialize)]
pub struct AttackSpec {
    pub damage: f64,
    pub range: RangeSpec,
    pub modifiers: HashMap<String, f64>,
}

#[derive(Deserialize)]
pub struct DefenseSpec {
    pub defense: f64,
    pub class: String,
}

pub type MovementClassSpec = HashMap<String, u32>;

#[derive(Deserialize)]
pub struct MovementSpec {
    pub movement: u32,
    pub class: String,
}

#[derive(Deserialize)]
pub struct UnitKindSpec {
    pub attack: AttackSpec,
    pub defense: DefenseSpec,
    pub movement: MovementSpec,
    pub sprite: SpriteSpec,
}

#[derive(Deserialize)]
pub struct Spec {
    pub movement_classes: HashMap<String, MovementClassSpec>,
    pub unit_kinds: HashMap<String, UnitKindSpec>,
    pub terrain: HashMap<String, TerrainSpec>,
    pub defense_classes: HashSet<String>,
}

pub type LayerSpec = HashMap<String, BTreeSet<(i32, i32, u32)>>;

#[derive(Deserialize)]
pub struct LevelSpec {
    pub name: String,
    pub schema: String,
    pub layers: HashMap<String, LayerSpec>,
}
