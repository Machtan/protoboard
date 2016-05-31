
#[derive(Debug, Clone)]
pub struct Unit {
    pub texture: &'static str,
    pub spent: bool,
    pub attack: AttackType,
}

impl Unit {
    #[inline]
    pub fn new(texture: &'static str, attack_type: AttackType) -> Unit {
        Unit {
            texture: texture,
            spent: false,
            attack: attack_type,
        }
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
    pub fn cells_in_range(&self, col: u32, row: u32, n_tiles: (u32, u32)) -> Vec<(u32, u32)> {
        let mut cells = Vec::new();
        let (n_cols, n_rows) = n_tiles;
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
