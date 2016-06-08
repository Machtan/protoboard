#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Terrain {
    pub name: String,
    pub defense: f64,
    pub texture: Option<String>,
}
