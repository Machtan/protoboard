
use common::{Message, State};

#[derive(Debug, Clone)]
pub struct Unit {
    pub texture: &'static str,
}

impl Unit {
    pub fn new(texture: &'static str) -> Self {
        Unit {
            texture: texture,
        }
    }
}