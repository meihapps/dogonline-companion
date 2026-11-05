use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionData {
    pub foods: Vec<Food>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Food {
    pub name: String,
    /// `None` where the wiki doesn't list a type.
    pub kind: Option<FoodType>,
    pub nutrition: u8,
    /// Mutable progress. Never part of the JSON data; filled in from browser storage.
    #[serde(skip)]
    pub eaten: bool,
}

impl NutritionData {
    /// Marks every food whose name is in `eaten` as eaten.
    pub fn apply_progress(&mut self, eaten: &HashSet<String>) {
        for food in &mut self.foods {
            food.eaten = eaten.contains(&food.name);
        }
    }

    /// Names of all eaten foods (what gets written to browser storage).
    pub fn eaten_names(&self) -> HashSet<String> {
        self.foods
            .iter()
            .filter(|food| food.eaten)
            .map(|food| food.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodType {
    Dairy,
    Fish,
    Fruit,
    Meat,
    Other,
}
