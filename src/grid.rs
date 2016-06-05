use std::collections::{btree_map, BTreeMap, BTreeSet};
use std::fmt::{self, Debug};
use std::mem;

use rand::{thread_rng, Rng};

use unit::{AttackType, Unit};

#[derive(Clone, Debug)]
pub enum Terrain {
    Grass,
    Woods,
    Mountains,
}

#[derive(Clone)]
pub struct Grid {
    size: (u32, u32),
    units: Box<[Option<Unit>]>,
    terrain: Box<[Terrain]>,
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
    pub fn tile_mut(&mut self, pos: (u32, u32)) -> (Option<&mut Unit>, &Terrain) {
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

    pub fn remove_unit(&mut self, pos: (u32, u32)) -> Unit {
        let slot = &mut self.units[self.index(pos)];
        mem::replace(slot, None).expect("no unit to remove")
    }

    pub fn move_unit(&mut self, from: (u32, u32), to: (u32, u32)) {
        let unit = self.units[self.index(from)].take();
        let dst = &mut self.units[self.index(to)];
        assert!(dst.is_none());
        *dst = unit;
    }

    pub fn find_attackable_before_moving<'a>(&'a self,
                                             unit: &'a Unit,
                                             pos: (u32, u32))
                                             -> FindAttackable<'a> {
        FindAttackable {
            unit: unit,
            grid: self,
            range: self.attack_range_before_moving(unit, pos),
        }
    }

    pub fn find_attackable_after_moving<'a>(&'a self,
                                            unit: &'a Unit,
                                            pos: (u32, u32))
                                            -> FindAttackable<'a> {
        FindAttackable {
            unit: unit,
            grid: self,
            range: self.attack_range_after_moving(unit, pos),
        }
    }

    pub fn path_finder(&self, pos: (u32, u32)) -> PathFinder {
        let unit = self.unit(pos).expect("no unit to find path for");
        let mut to_be_searched = vec![(pos, 0u32)];
        let mut costs = BTreeMap::new();
        let (w, h) = self.size();

        while let Some((pos, cost)) = to_be_searched.pop() {
            match costs.entry(pos) {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(cost);
                }
                btree_map::Entry::Occupied(mut entry) => {
                    if *entry.get() > cost {
                        entry.insert(cost);
                    } else {
                        continue;
                    }
                }
            }

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

                let nx = pos.0 as i32 + dx;
                let ny = pos.1 as i32 + dy;

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

                let tcost = unit.terrain_cost(terrain);
                if tcost == 0 {
                    unimplemented!();
                }
                let ncost = cost.saturating_add(tcost);

                if ncost <= unit.unit_type().movement {
                    to_be_searched.push((npos, ncost));
                }
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

    pub fn attack_range_before_moving<'a>(&'a self,
                                          unit: &'a Unit,
                                          pos: (u32, u32))
                                          -> AttackRange<'a> {
        match unit.unit_type().attack {
            AttackType::Melee => AttackRange::melee(self, pos),
            AttackType::Ranged { min, max } => AttackRange::ranged(self, pos, min, max),
            AttackType::Spear { range } => AttackRange::spear(self, unit, pos, range),
        }
    }

    pub fn attack_range_after_moving<'a>(&'a self,
                                         unit: &'a Unit,
                                         pos: (u32, u32))
                                         -> AttackRange<'a> {
        match unit.unit_type().attack {
            AttackType::Melee |
            AttackType::Spear { .. } => AttackRange::melee(self, pos),
            AttackType::Ranged { .. } => AttackRange::empty(),
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

#[derive(Clone)]
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

#[derive(Clone, Debug)]
pub struct PathFinder {
    origin: (u32, u32),
    // TODO: Should probably be private.
    pub costs: BTreeMap<(u32, u32), u32>,
}

impl PathFinder {
    pub fn total_attack_range(&self, grid: &Grid) -> BTreeSet<(u32, u32)> {
        let unit = grid.unit(self.origin).expect("no unit to find attackable targets for");

        let mut set = BTreeSet::new();

        set.extend(grid.attack_range_before_moving(unit, self.origin));

        // This is probably a somewhat ineffecient algorithm.
        for &pos in self.costs.keys() {
            if grid.unit(pos).is_some() {
                continue;
            }
            set.extend(grid.attack_range_after_moving(unit, pos));
        }
        set
    }

    #[inline]
    pub fn random_path_rev(&self, target: (u32, u32)) -> RandomPathRev {
        RandomPathRev {
            path_finder: self,
            pos: target,
        }
    }
}

#[derive(Clone)]
pub struct RandomPathRev<'a> {
    path_finder: &'a PathFinder,
    pos: (u32, u32),
}

impl<'a> Iterator for RandomPathRev<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        if self.pos == self.path_finder.origin {
            return None;
        }

        let cost = *self.path_finder.costs.get(&self.pos).expect("invalid position");

        let mut rng = thread_rng();
        let mut adjacent = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        rng.shuffle(&mut adjacent);

        let mut res = None;
        let mut cost = cost;
        for &(dx, dy) in &adjacent {
            let x = self.pos.0 as i32 + dx;
            let y = self.pos.1 as i32 + dy;

            if x < 0 || y < 0 {
                continue;
            }
            let npos = (x as u32, y as u32);
            if let Some(&ncost) = self.path_finder.costs.get(&npos) {
                if ncost < cost {
                    res = Some(npos);
                    cost = ncost;
                }
            }
        }
        let item = self.pos;
        self.pos = res.expect("path finder somehow produced a local minimum!");
        Some(item)
    }
}

#[derive(Clone)]
pub enum AttackRange<'a> {
    Empty,
    Melee {
        grid: &'a Grid,
        pos: (u32, u32),
        state: u8,
    },
    Ranged {
        grid: &'a Grid,
        pos: (u32, u32),
        min: u32,
        cur: (i32, i32),
    },
    Spear {
        grid: &'a Grid,
        unit: &'a Unit,
        pos: (u32, u32),
        max: u32,
        state: u8,
        dist: u32,
    },
}

impl<'a> AttackRange<'a> {
    #[inline]
    pub fn empty() -> AttackRange<'a> {
        AttackRange::Empty
    }

    #[inline]
    pub fn melee(grid: &'a Grid, pos: (u32, u32)) -> AttackRange<'a> {
        AttackRange::Melee {
            grid: grid,
            pos: pos,
            state: 0,
        }
    }

    #[inline]
    pub fn ranged(grid: &'a Grid, pos: (u32, u32), min: u32, max: u32) -> AttackRange<'a> {
        AttackRange::Ranged {
            grid: grid,
            pos: pos,
            min: min,
            cur: (0, max as i32),
        }
    }

    #[inline]
    pub fn spear(grid: &'a Grid, unit: &'a Unit, pos: (u32, u32), max: u32) -> AttackRange<'a> {
        AttackRange::Spear {
            grid: grid,
            unit: unit,
            pos: pos,
            max: max,
            state: 0,
            dist: 0,
        }
    }

    fn melee_next(grid: &Grid, pos: (u32, u32), state: &mut u8) -> Option<(u32, u32)> {
        let (x, y) = pos;
        let (w, h) = grid.size();
        loop {
            let (dx, dy) = match *state {
                0 => (0, 1),
                1 => (1, 0),
                2 => (0, -1),
                3 => (-1, 0),
                _ => return None,
            };
            *state += 1;

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if 0 <= nx && nx < w as i32 && 0 <= ny && ny < h as i32 {
                return Some((nx as u32, ny as u32));
            }
        }
    }

    fn ranged_next(grid: &Grid,
                   pos: (u32, u32),
                   min: u32,
                   cur: &mut (i32, i32))
                   -> Option<(u32, u32)> {
        let (x, y) = pos;
        let (w, h) = grid.size();
        while *cur != (0, min as i32 - 1) {
            let (dx, dy) = *cur;

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            *cur = match (dx.signum(), dy.signum()) {
                (0, 1) | (1, 1) => (dx + 1, dy - 1), // N-E
                (1, 0) | (1, -1) => (dx - 1, dy - 1), // E-S
                (0, -1) | (-1, -1) => (dx - 1, dy + 1), // S-W
                (-1, 0) | (-1, 1) => {
                    // W-N
                    if dx == -1 {
                        (dx + 1, dy)
                    } else {
                        (dx + 1, dy + 1)
                    }
                }
                _ => unreachable!(),
            };

            if 0 <= nx && nx < w as i32 && 0 <= ny && ny < h as i32 {
                return Some((nx as u32, ny as u32));
            }
        }
        None
    }

    fn spear_next(grid: &Grid,
                  unit: &Unit,
                  pos: (u32, u32),
                  max: u32,
                  state: &mut u8,
                  dist: &mut u32)
                  -> Option<(u32, u32)> {
        let (x, y) = pos;
        let (w, h) = grid.size();
        loop {
            let (dx, dy) = match *state {
                0 => (0, 1),
                1 => (1, 0),
                2 => (0, -1),
                3 => (-1, 0),
                _ => return None,
            };
            if *dist >= max {
                *state += 1;
                *dist = 0;
                continue;
            }

            *dist += 1;

            let nx = x as i32 + dx * *dist as i32;
            let ny = y as i32 + dy * *dist as i32;

            if !(0 <= nx && nx < w as i32 && 0 <= ny && ny < h as i32) {
                continue;
            }
            let res = (nx as u32, ny as u32);

            if let Some(other) = grid.unit(res) {
                // TODO: Alliances? Neutrals?
                if other.faction != unit.faction {
                    *state += 1;
                    *dist = 0;
                }
            }
            return Some(res);
        }
    }
}

impl<'a> Iterator for AttackRange<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        match *self {
            AttackRange::Empty => None,
            AttackRange::Melee { grid, pos, ref mut state } => {
                AttackRange::melee_next(grid, pos, state)
            }
            AttackRange::Ranged { grid, pos, min, ref mut cur } => {
                AttackRange::ranged_next(grid, pos, min, cur)
            }
            AttackRange::Spear { grid, unit, pos, max, ref mut state, ref mut dist } => {
                AttackRange::spear_next(grid, unit, pos, max, state, dist)
            }
        }
    }
}

pub struct FindAttackable<'a> {
    unit: &'a Unit,
    grid: &'a Grid,
    range: AttackRange<'a>,
}

impl<'a> Iterator for FindAttackable<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        for pos in &mut self.range {
            if let Some(ref other) = self.grid.unit(pos) {
                // TODO: Alliances? Neutrals?
                if other.faction != self.unit.faction {
                    return Some(pos);
                }
            }
        }
        None
    }
}
