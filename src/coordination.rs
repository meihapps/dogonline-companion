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
    /// Empty (and `bait` is `None`) where the wiki doesn't say yet.
    pub location: Vec<FishingSpot>,
    pub time: Vec<TimeOfDay>,
    pub weather: Vec<Weather>,
    pub bait: Option<Bait>,
    pub coordination: Option<u8>,
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
    DogwoodsFishingHole,
    HauntedShip,
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

// Named as on the wiki, Bombastic Bait included.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bait {
    BucketOfWorms,
    MoundOfMinnows,
    CupOfCrickets,
    PouchOfPrawns,
    GoldenGrub,
    BucketOfRadioactiveWorms,
    BombasticBait,
    BalloonHook,
    WickedSkymetalHook,
    SkyTackle,
    PremuimAnimalFeed,
    SuperPremuimAnimalFeed,
    SuperUltraPremiumAnimalFeed,
    CandyCornGrub,
    GummyChum,
    SourPowderedHook,
    // Cages are rods rather than bait on the wiki, but are listed as the bait to use.
    Cage,
    PumpkinCage,
}

impl Bait {
    /// The coordination needed to use it, from the wiki (none listed counts as 0).
    pub fn required_coordination(self) -> u32 {
        match self {
            Self::BucketOfWorms | Self::Cage => 0,
            Self::MoundOfMinnows => 3,
            Self::CupOfCrickets | Self::PumpkinCage => 10,
            Self::CandyCornGrub => 15,
            Self::BalloonHook
            | Self::PremuimAnimalFeed
            | Self::SuperPremuimAnimalFeed
            | Self::SuperUltraPremiumAnimalFeed => 20,
            Self::PouchOfPrawns => 25,
            Self::GummyChum => 30,
            Self::WickedSkymetalHook => 40,
            Self::BucketOfRadioactiveWorms => 50,
            Self::SkyTackle | Self::SourPowderedHook => 60,
            Self::GoldenGrub => 70,
            Self::BombasticBait => 100,
        }
    }
}
