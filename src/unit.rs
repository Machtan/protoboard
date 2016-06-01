use std::rc::Rc;
use std::fmt::{self, Debug};
use sdl2::render::Texture;

#[derive(Clone)]
pub struct Unit {
    pub texture: Rc<Texture>,
    pub spent: bool,
    pub attack: AttackType,
}

impl Unit {
    #[inline]
    pub fn new(texture: Rc<Texture>, attack_type: AttackType) -> Unit {
        Unit {
            texture: texture,
            spent: false,
            attack: attack_type,
        }
    }
}

impl Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Unit {{ attack: {:?}, spent: {} }}",
               self.attack,
               self.spent)
    }
}

#[derive(Debug, Clone)]
pub enum AttackType {
    Melee,
    Ranged {
        min: u32,
        max: u32,
    },
    Spear {
        range: u32,
    },
}

impl AttackType {
    pub fn cells_in_range(&self, cell: (u32, u32), grid_size: (u32, u32)) -> Vec<(u32, u32)> {
        let mut cells = Vec::new();
        let (col, row) = cell;
        let (n_cols, n_rows) = grid_size;
        let max_col = n_cols - 1;
        let max_row = n_rows - 1;
        match *self {
            _ => {
                // Melee
                if col > 0 {
                    cells.push((col - 1, row));
                }
                if col < max_col {
                    cells.push((col + 1, row));
                }
                if row > 0 {
                    cells.push((col, row - 1));
                }
                if row < max_row {
                    cells.push((col, row + 1));
                }
            }
        }
        cells
    }
}
