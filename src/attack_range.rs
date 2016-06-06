use grid::Grid;
use unit::Unit;

#[derive(Clone)]
pub struct AttackRange<'a> {
    kind: Kind<'a>,
}

#[derive(Clone)]
enum Kind<'a> {
    Empty,
    Melee(Melee<'a>),
    Ranged(Ranged<'a>),
    Spear(Spear<'a>),
}

impl<'a> AttackRange<'a> {
    #[inline]
    pub fn empty() -> AttackRange<'a> {
        AttackRange { kind: Kind::Empty }
    }

    #[inline]
    pub fn melee(grid: &'a Grid, pos: (u32, u32)) -> AttackRange<'a> {
        AttackRange {
            kind: Kind::Melee(Melee {
                grid: grid,
                pos: pos,
                state: 0,
            }),
        }
    }

    #[inline]
    pub fn ranged(grid: &'a Grid, pos: (u32, u32), min: u32, max: u32) -> AttackRange<'a> {
        AttackRange {
            kind: Kind::Ranged(Ranged {
                grid: grid,
                pos: pos,
                min: min,
                cur: (0, max as i32),
            }),
        }
    }

    #[inline]
    pub fn spear(grid: &'a Grid, unit: &'a Unit, pos: (u32, u32), max: u32) -> AttackRange<'a> {
        AttackRange {
            kind: Kind::Spear(Spear {
                grid: grid,
                unit: unit,
                pos: pos,
                max: max,
                state: 0,
                dist: 0,
            }),
        }
    }
}

impl<'a> Iterator for AttackRange<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        match self.kind {
            Kind::Empty => None,
            Kind::Melee(ref mut it) => it.next(),
            Kind::Ranged(ref mut it) => it.next(),
            Kind::Spear(ref mut it) => it.next(),
        }
    }
}

#[derive(Clone)]
struct Melee<'a> {
    grid: &'a Grid,
    pos: (u32, u32),
    state: u8,
}

impl<'a> Iterator for Melee<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let (x, y) = self.pos;
        let (w, h) = self.grid.size();
        loop {
            let (dx, dy) = match self.state {
                0 => (0, 1),
                1 => (1, 0),
                2 => (0, -1),
                3 => (-1, 0),
                _ => return None,
            };
            self.state += 1;

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if 0 <= nx && nx < w as i32 && 0 <= ny && ny < h as i32 {
                return Some((nx as u32, ny as u32));
            }
        }
    }
}

#[derive(Clone)]
struct Ranged<'a> {
    grid: &'a Grid,
    pos: (u32, u32),
    min: u32,
    cur: (i32, i32),
}

impl<'a> Iterator for Ranged<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let (x, y) = self.pos;
        let (w, h) = self.grid.size();
        while self.cur != (0, self.min as i32 - 1) {
            let (dx, dy) = self.cur;

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            self.cur = match (dx.signum(), dy.signum()) {
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
}

#[derive(Clone)]
struct Spear<'a> {
    grid: &'a Grid,
    unit: &'a Unit,
    pos: (u32, u32),
    max: u32,
    state: u8,
    dist: u32,
}

impl<'a> Iterator for Spear<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        let (x, y) = self.pos;
        let (w, h) = self.grid.size();
        loop {
            let (dx, dy) = match self.state {
                0 => (0, 1),
                1 => (1, 0),
                2 => (0, -1),
                3 => (-1, 0),
                _ => return None,
            };
            if self.dist >= self.max {
                self.state += 1;
                self.dist = 0;
                continue;
            }
            self.dist += 1;

            let nx = x as i32 + dx * self.dist as i32;
            let ny = y as i32 + dy * self.dist as i32;

            if !(0 <= nx && nx < w as i32 && 0 <= ny && ny < h as i32) {
                continue;
            }
            let res = (nx as u32, ny as u32);

            if let Some(other) = self.grid.unit(res) {
                if !self.unit.can_spear_through(other) {
                    self.state += 1;
                    self.dist = 0;
                }
            }
            return Some(res);
        }
    }
}
