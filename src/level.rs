use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::Path;

use json;

use faction::Faction;
use grid::Grid;
use resources::{ARCHER_PATH, PROTECTOR_PATH, RACCOON_PATH, WARRIOR_PATH};
use terrain::Terrain;
use unit::{AttackKind, Unit, UnitKind};

pub static WARRIOR: UnitKind = UnitKind {
    name: "warrior",
    health: 5,
    attack: AttackKind::Melee,
    damage: 3,
    movement: 6,
    texture: WARRIOR_PATH,
};

pub static ARCHER: UnitKind = UnitKind {
    name: "archer",
    health: 4,
    attack: AttackKind::Ranged { min: 2, max: 3 },
    damage: 5,
    movement: 4,
    texture: ARCHER_PATH,
};

pub static RACCOON: UnitKind = UnitKind {
    name: "raccoon",
    health: 8,
    attack: AttackKind::Spear { range: 3 },
    damage: 4,
    movement: 4,
    texture: RACCOON_PATH,
};

pub static DEFENDER: UnitKind = UnitKind {
    name: "defender",
    health: 10,
    attack: AttackKind::Melee,
    damage: 1,
    movement: 5,
    texture: PROTECTOR_PATH,
};

pub type Layer = BTreeMap<String, BTreeSet<(i32, i32)>>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Level {
    pub name: String,
    pub schema: String,
    pub layers: BTreeMap<String, Layer>,
}

impl Level {
    #[inline]
    pub fn load<P>(path: P) -> json::Result<Level>
        where P: AsRef<Path>
    {
        json::from_reader(File::open(path)?)
    }

    #[inline]
    pub fn save<P>(&self, path: P) -> json::Result<()>
        where P: AsRef<Path>
    {
        json::to_writer(&mut File::create(path)?, self)
    }

    pub fn create_grid(&self) -> Grid {
        let mut min_x = i32::max_value();
        let mut max_x = i32::min_value();
        let mut min_y = i32::max_value();
        let mut max_y = i32::min_value();

        for layer in self.layers.values() {
            for positions in layer.values() {
                for pos in positions {
                    min_x = cmp::min(pos.0, min_x);
                    max_x = cmp::max(pos.0, max_x);
                    min_y = cmp::min(pos.1, min_y);
                    max_y = cmp::max(pos.1, max_y);
                }
            }
        }

        let w = (max_x - min_x + 1) as u32;
        let h = (max_y - min_y + 1) as u32;

        let mut grid = match self.layers.get("terrain") {
            Some(terrain) => {
                Grid::new((w, h), |(x, y)| {
                    let pos = (x as i32 + min_x, y as i32 + min_y);
                    for (tile, positions) in terrain {
                        if positions.contains(&pos) {
                            return match &tile[..] {
                                "grass" => Terrain::Grass,
                                "mountains" => Terrain::Mountains,
                                "woods" => Terrain::Woods,
                                _ => panic!("unrecognized terrain type {:?}", tile),
                            };
                        }
                    }
                    Terrain::Grass
                })
            }
            None => Grid::new((w, h), |_| Terrain::Grass),
        };

        for &(layer_name, faction) in &[("units_red", Faction::Red),
                                        ("units_blue", Faction::Blue)] {
            for (tile, positions) in &self.layers[layer_name] {
                let kind = match &tile[..] {
                    "warrior" => &WARRIOR,
                    "archer" => &ARCHER,
                    "defender" => &DEFENDER,
                    "raccoon" => &RACCOON,
                    _ => panic!("unrecognized unit type {:?}", tile),
                };
                for &(x, y) in positions {
                    let pos = ((x - min_x) as u32, (y - min_y) as u32);
                    let unit = Unit::new(kind, faction);
                    grid.add_unit(unit, pos);
                }
            }
        }
        grid
    }
}
