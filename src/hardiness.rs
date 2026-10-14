use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardinessData {
    pub items: Vec<HardinessItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardinessItem {
    pub name: String,
    pub item_type: ItemType,
    pub hardiness: u8,
    /// Mutable progress. Never part of the JSON data; filled in from browser storage.
    #[serde(skip)]
    pub harvested: bool,
}

impl HardinessData {
    /// Marks every item whose name is in `harvested` as harvested.
    pub fn apply_progress(&mut self, harvested: &HashSet<String>) {
        for item in &mut self.items {
            item.harvested = harvested.contains(&item.name);
        }
    }

    /// Names of all harvested items (what gets written to browser storage).
    pub fn harvested_names(&self) -> HashSet<String> {
        self.items
            .iter()
            .filter(|item| item.harvested)
            .map(|item| item.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Crop,
    Gem,
    Bug,
    Egg,
}
