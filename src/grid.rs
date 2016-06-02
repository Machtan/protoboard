use std::fmt::{self, Debug};
use std::mem;

use unit::{Unit, TilesInRange};

#[derive(Clone, Debug)]
pub enum Terrain {
    Grass,
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
    pub fn new(size: (u32, u32)) -> Grid {
        let count = size.0 as usize * size.1 as usize;
        Grid {
            size: size,
            units: vec![None; count].into_boxed_slice(),
            terrain: vec![Terrain::Grass; count].into_boxed_slice(),
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

    #[inline]
    pub fn swap_units(&mut self, a: (u32, u32), b: (u32, u32)) {
        let i = self.index(a);
        let j = self.index(b);
        self.units.swap(i, j);
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

    /// Finds the tiles that the unit at the given position can attack.
    pub fn tiles_in_range(&self, pos: (u32, u32)) -> TilesInRange {
        self.unit(pos).expect("No unit for range check!").tiles_in_attack_range(pos, self.size)
    }

    /// Finds tiles attackable by the given unit if moved to the given position.
    pub fn find_attackable<'a>(&'a self, unit: &'a Unit, pos: (u32, u32)) -> FindAttackable<'a> {
        FindAttackable {
            unit: unit,
            grid: self,
            tiles: unit.tiles_in_attack_range(pos, self.size),
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
