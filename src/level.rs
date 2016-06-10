use std::cmp::{self, Ord, Ordering, PartialOrd};
use std::collections::{BTreeSet, HashMap};

use spec::LevelSpec;

use faction::Faction;
use grid::Grid;
use info::GameInfo;
use unit::Unit;

#[derive(Clone, Copy, Debug)]
pub struct Point(pub i32, pub i32, pub u32);

impl PartialEq for Point {
    #[inline]
    fn eq(&self, other: &Point) -> bool {
        (self.0, self.1) == (other.0, other.1)
    }
}

impl Eq for Point {}

impl PartialOrd for Point {
    #[inline]
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    #[inline]
    fn cmp(&self, other: &Point) -> Ordering {
        (self.0, self.1).cmp(&(other.0, other.1))
    }
}

pub type Layer = HashMap<String, BTreeSet<Point>>;

#[derive(Clone, Debug)]
pub struct Level {
    pub name: String,
    pub schema: String,
    pub layers: HashMap<String, Layer>,
}

impl Level {
    pub fn from_spec(spec: LevelSpec) -> Result<Level, String> {
        let layers = spec.layers
            .into_iter()
            .map(|(k, v)| {
                let v = v.into_iter()
                    .map(|(k, v)| (k, v.into_iter().map(|v| Point(v.0, v.1, v.2)).collect()))
                    .collect();
                (k, v)
            })
            .collect();
        Ok(Level {
            name: spec.name,
            schema: spec.schema,
            layers: layers,
        })
    }

    pub fn create_grid(&self, info: &GameInfo) -> Grid {
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
                    let pos = Point(x as i32 + min_x, y as i32 + min_y, 0);
                    for (tile, positions) in layer {
                        if positions.contains(&pos) {
                            return match info.terrain.get(&tile[..]) {
                                Some(terrain) => terrain.clone(),
                                None => panic!("terrain not in info file: {:?}", tile),
                            };
                        }
                    }
                    info.terrain["default"].clone()
                })
            }
            None => Grid::new((w, h), |_| info.terrain["default"].clone()),
        };

        for (tile, positions) in &self.layers["units"] {
            let kind = match info.unit_kinds.get(&tile[..]) {
                Some(kind) => kind,
                None => panic!("unit kind not in info file: {:?}", tile),
            };
            for &Point(x, y, color) in positions {
                let faction = match color {
                    1 => Faction::Red,
                    2 => Faction::Blue,
                    _ => panic!("invalid unit color index: {}", color),
                };
                let pos = ((x - min_x) as u32, (y - min_y) as u32);
                let unit = Unit::new(kind.clone(), faction);
                grid.add_unit(unit, pos);
            }
        }
        grid
    }
}
