#[derive(Debug, Clone)]
pub struct Unit {
    pub texture: &'static str,
}

impl Unit {
    #[inline]
    pub fn new(texture: &'static str) -> Unit {
        Unit { texture: texture }
    }
}
