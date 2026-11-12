//! Browser (localStorage) persistence for the mutable progress data.
//!
//! The JSON files hold the immutable game data. Which fish are caught and which
//! enemies are killed (and so on) is stored here as a JSON array of names, and merged into
//! `CoordinationData` / `BraveryData` at startup via `apply_progress`.

use std::collections::HashSet;

const CAUGHT_FISH_KEY: &str = "dogonline_companion.caught_fish";
const DAILIES_KEY: &str = "dogonline_companion.dailies";
const HARVESTED_ITEMS_KEY: &str = "dogonline_companion.harvested_items";
const EATEN_FOODS_KEY: &str = "dogonline_companion.eaten_foods";
const KILLED_ENEMIES_KEY: &str = "dogonline_companion.killed_enemies";
const READ_BOOKS_KEY: &str = "dogonline_companion.read_books";

/// Progress is stored by name, so names changed to match the wiki map (old, new) here to
/// keep what people had already marked.
const RENAMED_ENEMIES: [(&str, &str); 3] = [
    ("Pupcup", "Pup Cup"),
    ("Cheese", "Cheese (Enemy)"),
    ("Grow'd Up Pieder", "Growed-Up Pieder"),
];
const RENAMED_ITEMS: [(&str, &str); 1] = [("Golden Secretegg", "Gold Secretegg")];
const RENAMED_FISH: [(&str, &str); 1] = [("Rainbow Trout (Sky)", "Double Rainbow Trout")];
const RENAMED_FOODS: [(&str, &str); 1] = [("Rainbow Trout (Sky)", "Double Rainbow Trout")];

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn load_names(key: &str) -> HashSet<String> {
    local_storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|raw| serde_json::from_str::<HashSet<String>>(&raw).ok())
        .unwrap_or_default()
}

/// The stored value keeps the old names until the next save, which writes the new ones.
fn apply_renames(mut names: HashSet<String>, renames: &[(&str, &str)]) -> HashSet<String> {
    for (old, new) in renames {
        if names.remove(*old) {
            names.insert((*new).to_string());
        }
    }

    names
}

fn save_names(key: &str, names: &HashSet<String>) {
    let Some(storage) = local_storage() else {
        return;
    };

    // Sorted so the stored value is stable between saves.
    let mut sorted: Vec<&String> = names.iter().collect();
    sorted.sort();

    if let Ok(raw) = serde_json::to_string(&sorted) {
        // Storage can be full or disabled (private mode); progress just won't persist then.
        let _ = storage.set_item(key, &raw);
    }
}

fn load_string(key: &str) -> Option<String> {
    local_storage().and_then(|storage| storage.get_item(key).ok().flatten())
}

fn save_string(key: &str, value: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(key, value);
    }
}

/// UI filter settings, remembered between visits.
#[derive(Clone, Copy)]
pub enum Filter {
    Location,
    Time,
    Weather,
    StandardBait,
    HookBait,
    FeedBait,
    GoldenGrub,
    ShowCaught,
    ShowKilled,
    ShowHarvested,
    ShowEaten,
    ShowRead,
    ShowZeroNutrition,
    FoodType,
}

impl Filter {
    fn key(self) -> &'static str {
        match self {
            Self::Location => "dogonline_companion.filter.location",
            Self::Time => "dogonline_companion.filter.time",
            Self::Weather => "dogonline_companion.filter.weather",
            Self::StandardBait => "dogonline_companion.filter.standard_bait",
            Self::HookBait => "dogonline_companion.filter.hook_bait",
            Self::FeedBait => "dogonline_companion.filter.feed_bait",
            Self::GoldenGrub => "dogonline_companion.filter.golden_grub",
            Self::ShowCaught => "dogonline_companion.filter.show_caught",
            Self::ShowKilled => "dogonline_companion.filter.show_killed",
            Self::ShowHarvested => "dogonline_companion.filter.show_harvested",
            Self::ShowEaten => "dogonline_companion.filter.show_eaten",
            Self::ShowRead => "dogonline_companion.filter.show_read",
            Self::ShowZeroNutrition => "dogonline_companion.filter.show_zero_nutrition",
            Self::FoodType => "dogonline_companion.filter.food_type",
        }
    }
}

pub fn load_filter_string(filter: Filter, default: &str) -> String {
    load_string(filter.key()).unwrap_or_else(|| default.to_string())
}

pub fn save_filter_string(filter: Filter, value: &str) {
    save_string(filter.key(), value);
}

pub fn load_filter_bool(filter: Filter, default: bool) -> bool {
    load_string(filter.key()).map_or(default, |raw| raw == "true")
}

pub fn save_filter_bool(filter: Filter, value: bool) {
    save_string(filter.key(), if value { "true" } else { "false" });
}

pub fn load_caught_fish() -> HashSet<String> {
    apply_renames(load_names(CAUGHT_FISH_KEY), &RENAMED_FISH)
}

pub fn save_caught_fish(names: &HashSet<String>) {
    save_names(CAUGHT_FISH_KEY, names);
}

pub fn load_killed_enemies() -> HashSet<String> {
    apply_renames(load_names(KILLED_ENEMIES_KEY), &RENAMED_ENEMIES)
}

pub fn save_killed_enemies(names: &HashSet<String>) {
    save_names(KILLED_ENEMIES_KEY, names);
}

/// Ticked-off dailies. Stored with the (Pacific) day they were ticked, so a new day
/// reads back as empty.
pub fn load_dailies(today: &str) -> HashSet<String> {
    let Some(raw) = load_string(DAILIES_KEY) else {
        return HashSet::new();
    };

    serde_json::from_str::<(String, HashSet<String>)>(&raw)
        .ok()
        .filter(|(day, _)| day == today)
        .map(|(_, done)| done)
        .unwrap_or_default()
}

pub fn save_dailies(today: &str, done: &HashSet<String>) {
    // Sorted so the stored value is stable between saves.
    let mut sorted: Vec<&String> = done.iter().collect();
    sorted.sort();

    if let Ok(raw) = serde_json::to_string(&(today, sorted)) {
        save_string(DAILIES_KEY, &raw);
    }
}

pub fn load_harvested_items() -> HashSet<String> {
    apply_renames(load_names(HARVESTED_ITEMS_KEY), &RENAMED_ITEMS)
}

pub fn save_harvested_items(names: &HashSet<String>) {
    save_names(HARVESTED_ITEMS_KEY, names);
}

pub fn load_eaten_foods() -> HashSet<String> {
    apply_renames(load_names(EATEN_FOODS_KEY), &RENAMED_FOODS)
}

pub fn save_eaten_foods(names: &HashSet<String>) {
    save_names(EATEN_FOODS_KEY, names);
}

pub fn load_read_books() -> HashSet<String> {
    load_names(READ_BOOKS_KEY)
}

pub fn save_read_books(names: &HashSet<String>) {
    save_names(READ_BOOKS_KEY, names);
}

/// Manual colour-scheme override. `Device` follows the OS/browser setting.
#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Device,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

const THEME_KEY: &str = "dogonline_companion.theme";

pub fn load_theme() -> Theme {
    match load_string(THEME_KEY).as_deref() {
        Some("light") => Theme::Light,
        Some("dark") => Theme::Dark,
        _ => Theme::Device,
    }
}

pub fn save_theme(theme: Theme) {
    save_string(THEME_KEY, theme.as_str());
}

/// The page that was open last, reopened on the next visit.
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Coordination,
    Creativity,
    Nutrition,
    Bravery,
    Hardiness,
    Dailies,
}

impl Tab {
    fn as_str(self) -> &'static str {
        match self {
            Self::Coordination => "coordination",
            Self::Creativity => "creativity",
            Self::Nutrition => "nutrition",
            Self::Bravery => "bravery",
            Self::Hardiness => "hardiness",
            Self::Dailies => "dailies",
        }
    }
}

const TAB_KEY: &str = "dogonline_companion.tab";

pub fn load_tab() -> Tab {
    match load_string(TAB_KEY).as_deref() {
        Some("creativity") => Tab::Creativity,
        Some("nutrition") => Tab::Nutrition,
        Some("bravery") => Tab::Bravery,
        Some("hardiness") => Tab::Hardiness,
        Some("dailies") => Tab::Dailies,
        _ => Tab::Coordination,
    }
}

pub fn save_tab(tab: Tab) {
    save_string(TAB_KEY, tab.as_str());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> HashSet<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn apply_renames_only_changes_old_names() {
        assert_eq!(
            apply_renames(names(&["Pupcup", "Peeper"]), &RENAMED_ENEMIES),
            names(&["Pup Cup", "Peeper"])
        );
        assert_eq!(
            apply_renames(names(&["Pup Cup"]), &RENAMED_ENEMIES),
            names(&["Pup Cup"])
        );
    }
}
