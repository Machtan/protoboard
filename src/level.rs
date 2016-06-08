use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Write as FmtWrite};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::rc::Rc;

use json;
use serde::Deserialize;
use toml;

use faction::Faction;
use grid::Grid;
use terrain::Terrain;
use unit::{AttackKind, Unit, UnitKind};

#[derive(Clone, Debug, Deserialize, Serialize)]
enum TextureSpec {
    Texture(String),
    Sprite(String, u32, u32, u32, u32),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UnitSpec {
    texture: TextureSpec,
    damage: f64,
    defense: f64,
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
        let (texture, area) = match self.texture.clone() {
            TextureSpec::Texture(path) => (path, None),
            TextureSpec::Sprite(path, x, y, w, h) => (path, Some((x, y, w, h))),
        };
        Ok(UnitKind {
            name: name,
            damage: self.damage,
            defense: self.defense,
            movement: self.movement,
            attack: self.attack.to_kind()?,
            texture: texture,
            texture_area: area,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InfoFile {
    units: BTreeMap<String, UnitSpec>,
    terrain: BTreeMap<String, Terrain>,
}

#[derive(Debug)]
pub enum LoadError {
    Read(io::Error),
    Parse(toml::ParserError, String),
    Decode(toml::DecodeError),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::Read(ref err) => write!(f, "error reading file: {}", err),
            LoadError::Parse(ref err, ref ctx) => write!(f, "error parsing file: {}\n{}", err, ctx),
            LoadError::Decode(ref err) => write!(f, "error decoding file: {}", err),
        }
    }
}

fn warn_about_unused_keys<F>(value: toml::Value, path: &mut String, warn: &mut F) -> bool
    where F: FnMut(&str)
{
    let mut result = false;
    if let toml::Value::Table(table) = value {
        for (key, child) in table {
            result = true;
            let i = path.len();
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(&key);
            if !warn_about_unused_keys(child, path, warn) {
                warn(&format!("unused key in info file: {:?}", path));
            }
            path.truncate(i);
        }
    }
    result
}

impl InfoFile {
    pub fn load<P, F>(path: P, mut warn: F) -> Result<InfoFile, LoadError>
        where P: AsRef<Path>,
              F: FnMut(&str)
    {
        let mut contents = String::new();
        File::open(path)
            .and_then(|mut file| file.read_to_string(&mut contents))
            .map_err(LoadError::Read)?;
        let mut parser = toml::Parser::new(&contents);
        let table = match parser.parse() {
            Some(table) => Ok(table),
            None => {
                let err = parser.errors.pop().unwrap();

                let i = contents[..err.lo + 1].rfind('\n').unwrap_or(!0).wrapping_add(1);
                let j = err.lo + contents[err.lo..].find('\n').unwrap_or(contents.len() - err.lo);
                let line_num = contents[..i].split('\n').count() + 1;
                let line = &contents[i..j];

                let trailing = if j < err.hi {
                    "..."
                } else {
                    ""
                };
                let mut ctx = format!("{} > ", line_num);
                let prefix_len = ctx.len();
                write!(ctx, "{}{}\n", line, trailing);
                for _ in 0..(prefix_len + err.lo - i) {
                    ctx.push(' ');
                }
                for _ in 0..cmp::max(cmp::min(err.hi - err.lo, j - err.lo), 1) {
                    ctx.push('~');
                }

                Err(LoadError::Parse(err, ctx))
            }
        };
        for warning in &parser.errors {
            warn(&format!("info file: {}", warning));
        }
        let table = table?;

        let mut decoder = toml::Decoder::new(toml::Value::Table(table));
        let info = InfoFile::deserialize(&mut decoder).map_err(LoadError::Decode)?;

        if let Some(value) = decoder.toml {
            let mut path = String::new();
            warn_about_unused_keys(value, &mut path, &mut warn);
        }

        Ok(info)
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
