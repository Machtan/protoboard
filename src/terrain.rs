#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Terrain {
    pub cost: u32,
    pub defense: f64,
    pub texture: Option<String>,
}
