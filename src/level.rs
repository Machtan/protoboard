use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use json;
use toml;

use faction::Faction;
use grid::Grid;
use terrain::Terrain;
use unit::{AttackKind, Unit, UnitKind};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UnitSpec {
    texture: String,
    health: u32,
    damage: u32,
    movement: u32,
    attack: AttackSpec,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct AttackSpec {
    kind: String,
    min: Option<u32>,
    max: Option<u32>,
    range: Option<u32>,
}

impl AttackSpec {
    fn to_kind(&self) -> Result<AttackKind, String> {
        Ok(match &self.kind[..] {
            "melee" => AttackKind::Melee,
            "ranged" => {
                AttackKind::Ranged {
                    min: self.min.ok_or_else(|| format!("ranged attack missing field: min"))?,
                    max: self.max.ok_or_else(|| format!("ranged attack missing field: max"))?,
                }
            }
            "spear" => {
                AttackKind::Spear {
                    range: self.range.ok_or_else(|| format!("spear attack missing field: range"))?,
                }
            }
            k => return Err(format!("unrecognized attack kind {:?}", k)),
        })
    }
}

impl UnitSpec {
    fn to_kind(&self, name: String) -> Result<UnitKind, String> {
        Ok(UnitKind {
            name: name,
            health: self.health,
            damage: self.damage,
            movement: self.movement,
            attack: self.attack.to_kind()?,
            texture: self.texture.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InfoFile {
    units: BTreeMap<String, UnitSpec>,
    terrain: BTreeMap<String, Terrain>,
}

impl InfoFile {
    pub fn load<P>(path: P) -> Option<InfoFile>
        where P: AsRef<Path>
    {
        let mut contents = String::new();
        let res = File::open(path).and_then(|mut file| file.read_to_string(&mut contents));
        if let Err(err) = res {
            error!("could not load info file: {}", err);
            return None;
        }
        toml::decode_str(&contents)
    }
}

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

    pub fn create_grid(&self, info: &InfoFile) -> Grid {
        let kinds = info.units
            .iter()
            .map(|(name, spec)| (&name[..], Rc::new(spec.to_kind(name.to_owned()).unwrap())))
            .collect::<BTreeMap<_, _>>();

        let terrain = info.terrain
            .iter()
            .map(|(name, spec)| (&name[..], Rc::new(spec.clone())))
            .collect::<BTreeMap<_, _>>();

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
            Some(layer) => {
                Grid::new((w, h), |(x, y)| {
                    let pos = (x as i32 + min_x, y as i32 + min_y);
                    for (tile, positions) in layer {
                        if positions.contains(&pos) {
                            return match terrain.get(&tile[..]) {
                                Some(terrain) => terrain.clone(),
                                None => panic!("terrain not in info file: {:?}", tile),
                            };
                        }
                    }
                    terrain["default"].clone()
                })
            }
            None => Grid::new((w, h), |_| terrain["default"].clone()),
        };

        for &(layer_name, faction) in &[("units_red", Faction::Red),
                                        ("units_blue", Faction::Blue)] {
            for (tile, positions) in &self.layers[layer_name] {
                let kind = match kinds.get(&tile[..]) {
                    Some(kind) => kind,
                    None => panic!("unit kind not in info file: {:?}", tile),
                };
                for &(x, y) in positions {
                    let pos = ((x - min_x) as u32, (y - min_y) as u32);
                    let unit = Unit::new(kind.clone(), faction);
                    grid.add_unit(unit, pos);
                }
            }
        }
        grid
    }
}
