use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationData {
    pub fish: Vec<Fish>,
}

impl CoordinationData {
    /// Marks every fish whose name is in `caught` as caught.
    pub fn apply_progress(&mut self, caught: &HashSet<String>) {
        for fish in &mut self.fish {
            fish.caught = caught.contains(&fish.name);
        }
    }

    /// Names of all caught fish (what gets written to browser storage).
    pub fn caught_names(&self) -> HashSet<String> {
        self.fish
            .iter()
            .filter(|fish| fish.caught)
            .map(|fish| fish.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fish {
    pub name: String,
    pub location: Vec<FishingSpot>,
    pub time: Vec<TimeOfDay>,
    pub weather: Vec<Weather>,
    pub bait: Bait,
    pub coordination: u8,
    /// Mutable progress. Never part of the JSON data; filled in from browser storage.
    #[serde(skip)]
    pub caught: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FishingSpot {
    OldFishingHole,
    DesertOasis,
    IceFishingHole,
    OceanFishingCruiser,
    FlyFishingShip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Weather {
    ClearSkies,
    Rainy,
    Stormy,
    Snowy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bait {
    BucketOfWorms,
    MoundOfMinnows,
    CupOfCrickets,
    PouchOfPrawns,
    GoldenGrub,
    BalloonHook,
    WickedSkymetalHook,
    SkyTackle,
}
