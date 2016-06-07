use std::collections::{btree_map, BTreeMap, BTreeSet};
use std::fmt::{self, Debug};
use std::mem;
use std::rc::Rc;

use rand::{thread_rng, Rng};

use attack_range::AttackRange;
use terrain::Terrain;
use unit::{AttackKind, Unit};

#[derive(Clone)]
pub struct Grid {
    size: (u32, u32),
    units: Box<[Option<Unit>]>,
    terrain: Box<[Rc<Terrain>]>,
}

impl Grid {
    pub fn new<F>(size: (u32, u32), mut func: F) -> Grid
        where F: FnMut((u32, u32)) -> Rc<Terrain>
    {
        let count = size.0 as usize * size.1 as usize;
        let terrain = (0..count)
            .map(|i| {
                let x = (i % size.0 as usize) as u32;
                let y = (i / size.0 as usize) as u32;
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
    fn index(&self, pos: (u32, u32)) -> usize {
        let (x, y) = pos;
        let (w, h) = self.size;
        assert!(x < w && y < h);
        y as usize * w as usize + x as usize
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
    pub fn terrain(&self, pos: (u32, u32)) -> &Terrain {
        let i = self.index(pos);
        &self.terrain[i]
    }

    #[inline]
    pub fn unit(&self, pos: (u32, u32)) -> Option<&Unit> {
        self.tile(pos).0
    }

    #[inline]
    pub fn unit_mut(&mut self, pos: (u32, u32)) -> Option<&mut Unit> {
        self.tile_mut(pos).0
    }

    #[inline]
    pub fn units(&self) -> Units {
        Units { units: &self.units[..] }
    }

    #[inline]
    pub fn units_mut(&mut self) -> UnitsMut {
        UnitsMut { units: &mut self.units[..] }
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

    pub fn attack_range_before_moving<'a>(&'a self,
                                          unit: &'a Unit,
                                          pos: (u32, u32))
                                          -> AttackRange<'a> {
        match unit.kind().attack {
            AttackKind::Melee => AttackRange::melee(self, pos),
            AttackKind::Ranged { min, max } => AttackRange::ranged(self, pos, min, max),
            AttackKind::Spear { range } => AttackRange::spear(self, unit, pos, range),
        }
    }

    pub fn attack_range_after_moving<'a>(&'a self,
                                         unit: &'a Unit,
                                         pos: (u32, u32))
                                         -> AttackRange<'a> {
        match unit.kind().attack {
            AttackKind::Melee |
            AttackKind::Spear { .. } => AttackRange::melee(self, pos),
            AttackKind::Ranged { .. } => AttackRange::empty(),
        }
    }

    pub fn attack_range_when_retaliating<'a>(&'a self,
                                             unit: &'a Unit,
                                             pos: (u32, u32))
                                             -> AttackRange<'a> {
        self.attack_range_before_moving(unit, pos)
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

                if let Some(other) = other {
                    if !unit.can_move_through(other) {
                        continue;
                    }
                }

                let tcost = unit.terrain_cost(terrain);
                if tcost == 0 {
                    unimplemented!();
                }
                let ncost = cost.saturating_add(tcost);

                if ncost <= unit.kind().movement {
                    to_be_searched.push((npos, ncost));
                }
            }
        }
        PathFinder {
            origin: pos,
            costs: costs,
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
    costs: BTreeMap<(u32, u32), u32>,
}

impl PathFinder {
    #[inline]
    pub fn can_move_to(&self, pos: (u32, u32)) -> bool {
        self.costs.contains_key(&pos)
    }

    #[inline]
    pub fn cost(&self, pos: (u32, u32)) -> Option<u32> {
        self.costs.get(&pos).cloned()
    }

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

        let cost = self.path_finder.cost(self.pos).expect("invalid position");

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
            if let Some(ncost) = self.path_finder.cost(npos) {
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
                if self.unit.can_attack(other) {
                    return Some(pos);
                }
            }
        }
        None
    }
}
