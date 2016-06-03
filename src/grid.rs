use std::collections::{btree_map, BTreeMap, BTreeSet};
use std::fmt::{self, Debug};
use std::mem;

use unit::{TilesInRange, Unit};

#[derive(Clone, Debug)]
pub enum Terrain {
    Grass,
    Woods,
    Mountain,
}

#[derive(Clone)]
pub struct Grid {
    size: (u32, u32),
    units: Box<[Option<Unit>]>,
    terrain: Box<[Terrain]>,
}

pub struct Units<'a> {
    units: &'a [Option<Unit>],
}

impl<'a> Iterator for Units<'a> {
    type Item = &'a Unit;

    fn next(&mut self) -> Option<&'a Unit> {
        loop {
            match self.units.split_first() {
                Some((first, rest)) => {
                    self.units = rest;
                    if let Some(ref unit) = *first {
                        return Some(unit);
                    }
                }
                None => return None,
            }
        }
    }
}

pub struct UnitsMut<'a> {
    units: &'a mut [Option<Unit>],
}

impl<'a> Iterator for UnitsMut<'a> {
    type Item = &'a mut Unit;

    fn next(&mut self) -> Option<&'a mut Unit> {
        loop {
            let slice = mem::replace(&mut self.units, &mut []);
            match slice.split_first_mut() {
                Some((first, rest)) => {
                    self.units = rest;
                    if let Some(ref mut unit) = *first {
                        return Some(unit);
                    }
                }
                None => return None,
            }
        }
    }
}


impl Grid {
    pub fn new<F>(size: (u32, u32), mut func: F) -> Grid
        where F: FnMut((u32, u32)) -> Terrain
    {
        let count = size.0 as usize * size.1 as usize;
        let terrain = (0..count)
            .map(|i| {
                let x = (i % size.1 as usize) as u32;
                let y = (i / size.1 as usize) as u32;
                func((x, y))
            })
            .collect::<Vec<_>>();
        Grid {
            size: size,
            units: vec![None; count].into_boxed_slice(),
            terrain: terrain.into_boxed_slice(),
        }
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    #[inline]
    pub fn tile(&self, pos: (u32, u32)) -> (Option<&Unit>, &Terrain) {
        let i = self.index(pos);
        (self.units[i].as_ref(), &self.terrain[i])
    }

    #[inline]
    pub fn tile_mut(&mut self, pos: (u32, u32)) -> (Option<&mut Unit>, &mut Terrain) {
        let i = self.index(pos);
        (self.units[i].as_mut(), &mut self.terrain[i])
    }

    #[inline]
    pub fn units(&self) -> Units {
        Units { units: &self.units[..] }
    }

    #[inline]
    pub fn units_mut(&mut self) -> UnitsMut {
        UnitsMut { units: &mut self.units[..] }
    }

    #[inline]
    pub fn unit(&self, pos: (u32, u32)) -> Option<&Unit> {
        self.tile(pos).0
    }

    #[inline]
    pub fn unit_mut(&mut self, pos: (u32, u32)) -> Option<&mut Unit> {
        self.tile_mut(pos).0
    }

    /// Adds a unit to the grid.
    pub fn add_unit(&mut self, unit: Unit, pos: (u32, u32)) {
        let slot = &mut self.units[self.index(pos)];
        assert!(slot.is_none());
        *slot = Some(unit);
    }

    pub fn remove_unit(&mut self, pos: (u32, u32)) {
        let slot = &mut self.units[self.index(pos)];
        assert!(slot.is_some());
        *slot = None;
    }

    pub fn move_unit(&mut self, from: (u32, u32), to: (u32, u32)) {
        let unit = self.units[self.index(from)].take();
        let dst = &mut self.units[self.index(to)];
        assert!(dst.is_none());
        *dst = unit;
    }

    pub fn unit_pair_mut(&mut self,
                         a: (u32, u32),
                         b: (u32, u32))
                         -> Option<(Option<&mut Unit>, Option<&mut Unit>)> {
        if a == b {
            return None;
        }

        let i = self.index(a);
        let j = self.index(b);

        let (i, j, swapped) = if i < j {
            (i, j, false)
        } else {
            (j, i, true)
        };

        if let Some((last, rest)) = self.units[..j + 1].split_last_mut() {
            let first = &mut rest[i];
            if swapped {
                Some((last.as_mut(), first.as_mut()))
            } else {
                Some((first.as_mut(), last.as_mut()))
            }
        } else {
            unreachable!();
        }
    }

    /// Finds tiles attackable by the given unit if moved to the given position.
    pub fn find_attackable<'a>(&'a self, unit: &'a Unit, pos: (u32, u32)) -> FindAttackable<'a> {
        FindAttackable {
            unit: unit,
            grid: self,
            tiles: unit.tiles_in_attack_range(pos, self.size),
        }
    }

    pub fn path_finder(&self, pos: (u32, u32)) -> PathFinder {
        let unit = self.unit(pos).expect("no unit to find path for");
        let mut to_be_searched = vec![(pos, 0u32)];
        let mut costs = BTreeMap::new();
        let (w, h) = self.size();

        while let Some(((x, y), cost)) = to_be_searched.pop() {
            let mut dir = 0;
            loop {
                let (dx, dy) = match dir {
                    0 => (1, 0),
                    1 => (0, 1),
                    2 => (-1, 0),
                    3 => (0, -1),
                    _ => break,
                };
                dir += 1;

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx < 0 || w as i32 <= nx || ny < 0 || h as i32 <= ny {
                    continue;
                }

                let npos = (nx as u32, ny as u32);

                let (other, terrain) = self.tile(npos);

                // TODO: Alliances? Neutrals?
                if let Some(other) = other {
                    if other.faction != unit.faction {
                        continue;
                    }
                }

                let ncost = cost.saturating_add(unit.terrain_cost(terrain));

                if ncost > unit.unit_type().movement {
                    continue;
                }

                match costs.entry(npos) {
                    btree_map::Entry::Vacant(entry) => {
                        entry.insert(ncost);
                    }
                    btree_map::Entry::Occupied(mut entry) => {
                        if *entry.get() > ncost {
                            entry.insert(ncost);
                        } else {
                            continue;
                        }
                    }
                }
                to_be_searched.push((npos, ncost));
            }
        }
        PathFinder {
            origin: pos,
            costs: costs,
        }
    }

    #[inline]
    fn index(&self, pos: (u32, u32)) -> usize {
        let (x, y) = pos;
        let (w, h) = self.size;
        assert!(x < w && y < h);
        y as usize * w as usize + x as usize
    }
}

#[derive(Clone, Debug)]
pub struct PathFinder {
    origin: (u32, u32),
    // TODO: Should probably be private.
    pub costs: BTreeMap<(u32, u32), u32>,
}

impl PathFinder {
    pub fn find_all_attackable(&self, grid: &Grid) -> BTreeSet<(u32, u32)> {
        let unit = grid.unit(self.origin).expect("no unit to find attackable targets for");
        if unit.unit_type().attack.is_ranged() {
            grid.find_attackable(unit, self.origin).map(|(p, _)| p).collect()
        } else {
            // TODO: Somewhat ineffective algorithm.
            let mut res = BTreeSet::new();
            for &pos in self.costs.keys() {
                if grid.unit(pos).is_some() {
                    continue;
                }
                for (target, _) in grid.find_attackable(unit, pos) {
                    res.insert(target);
                }
            }
            res
        }
    }

    pub fn tiles_in_attack_range(&self, grid: &Grid) -> BTreeSet<(u32, u32)> {
        let unit = grid.unit(self.origin).expect("no unit to find attackable targets for");

        if unit.unit_type().attack.is_ranged() {
            unit.tiles_in_attack_range(self.origin, grid.size()).collect()
        } else {
            // TODO: Somewhat ineffective algorithm.
            let mut res = BTreeSet::new();
            for &pos in self.costs.keys() {
                if pos != self.origin && grid.unit(pos).is_some() {
                    continue;
                }
                res.extend(unit.tiles_in_attack_range(pos, grid.size()));
            }
            res
        }
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Grid")
            .field("size", &self.size)
            .field("units", &(..))
            .field("tiles", &(..))
            .finish()
    }
}

pub struct FindAttackable<'a> {
    unit: &'a Unit,
    grid: &'a Grid,
    tiles: TilesInRange,
}

impl<'a> Iterator for FindAttackable<'a> {
    type Item = ((u32, u32), &'a Unit);

    fn next(&mut self) -> Option<((u32, u32), &'a Unit)> {
        for pos in &mut self.tiles {
            if let Some(ref other) = self.grid.unit(pos) {
                // TODO: Alliances? Neutrals?
                if other.faction != self.unit.faction {
                    return Some((pos, other));
                }
            }
        }
        None
    }
}
