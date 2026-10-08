use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraveryData {
    pub enemies: Vec<Enemy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub damage_type: DamageType,
    pub hp: u32,
    pub bravery: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Melee,
    Ranged,
    Magic,
    Usable,
}
