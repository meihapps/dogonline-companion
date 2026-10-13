use std::collections::HashSet;

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
    /// Mutable progress. Never part of the JSON data; filled in from browser storage.
    #[serde(skip)]
    pub killed: bool,
}

impl BraveryData {
    /// Marks every enemy whose name is in `killed` as killed.
    pub fn apply_progress(&mut self, killed: &HashSet<String>) {
        for enemy in &mut self.enemies {
            enemy.killed = killed.contains(&enemy.name);
        }
    }

    /// Names of all killed enemies (what gets written to browser storage).
    pub fn killed_names(&self) -> HashSet<String> {
        self.enemies
            .iter()
            .filter(|enemy| enemy.killed)
            .map(|enemy| enemy.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Melee,
    Ranged,
    Magic,
    Usable,
}
