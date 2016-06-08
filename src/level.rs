use std::cmp;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Display, Write as FmtWrite};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use json;
use serde::Deserialize;
use toml;

use faction::Faction;
use grid::Grid;
use info::GameInfo;
use spec::Spec;
use unit::Unit;

#[derive(Debug)]
pub enum LoadError {
    Read(io::Error),
    Parse,
    Decode(toml::DecodeError),
    Validate(String),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::Read(ref err) => write!(f, "error reading file: {}", err),
            LoadError::Parse => write!(f, "error parsing file"),
            LoadError::Decode(ref err) => write!(f, "error decoding file: {}", err),
            LoadError::Validate(ref err) => write!(f, "error validating file: {}", err),
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

fn make_context(err: &toml::ParserError, contents: &str, ctx: &mut String) {
    let i = contents[..err.lo + 1].rfind('\n').unwrap_or(!0).wrapping_add(1);
    let j = err.lo + contents[err.lo..].find('\n').unwrap_or(contents.len() - err.lo);
    let line_num = contents[..i].split('\n').count() + 1;
    let line = &contents[i..j];

    let trailing = if j < err.hi {
        "..."
    } else {
        ""
    };
    let prev_len = ctx.len();
    write!(ctx, "{} > ", line_num).unwrap();
    let prefix_len = ctx.len() - prev_len;
    write!(ctx, "{}{}\n", line, trailing).unwrap();
    for _ in 0..(prefix_len + err.lo - i) {
        ctx.push(' ');
    }
    for _ in 0..cmp::max(cmp::min(err.hi - err.lo, j - err.lo), 1) {
        ctx.push('~');
    }
}

pub fn load_info<P, F>(path: P, mut warn: F) -> Result<GameInfo, LoadError>
    where P: AsRef<Path>,
          F: FnMut(&str)
{
    let mut contents = String::new();
    File::open(path)
        .and_then(|mut file| file.read_to_string(&mut contents))
        .map_err(LoadError::Read)?;
    let mut parser = toml::Parser::new(&contents);

    let table = parser.parse();
    for warning in &parser.errors {
        let mut msg = format!("parsing file: {}\n", warning);
        make_context(warning, &contents, &mut msg);
        warn(&msg);
    }
    let table = table.ok_or(LoadError::Parse)?;

    let mut decoder = toml::Decoder::new(toml::Value::Table(table));
    let spec = Spec::deserialize(&mut decoder).map_err(LoadError::Decode)?;

    if let Some(value) = decoder.toml {
        let mut path = String::new();
        warn_about_unused_keys(value, &mut path, &mut warn);
    }

    spec.to_info().map_err(LoadError::Validate)
}

pub type Layer = HashMap<String, BTreeSet<(i32, i32)>>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Level {
    pub name: String,
    pub schema: String,
    pub layers: HashMap<String, Layer>,
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
                    let pos = (x as i32 + min_x, y as i32 + min_y);
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

        for &(layer_name, faction) in &[("units_red", Faction::Red),
                                        ("units_blue", Faction::Blue)] {
            for (tile, positions) in &self.layers[layer_name] {
                let kind = match info.roles.get(&tile[..]) {
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
