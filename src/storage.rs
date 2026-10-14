//! Browser (localStorage) persistence for the mutable progress data.
//!
//! The JSON files hold the immutable game data. Which fish are caught and which
//! enemies are killed is stored here as a JSON array of names, and merged into
//! `CoordinationData` / `BraveryData` at startup via `apply_progress`.

use std::collections::HashSet;

const CAUGHT_FISH_KEY: &str = "dogonline_companion.caught_fish";
const DAILIES_KEY: &str = "dogonline_companion.dailies";
const HARVESTED_ITEMS_KEY: &str = "dogonline_companion.harvested_items";
const KILLED_ENEMIES_KEY: &str = "dogonline_companion.killed_enemies";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn load_names(key: &str) -> HashSet<String> {
    local_storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|raw| serde_json::from_str::<HashSet<String>>(&raw).ok())
        .unwrap_or_default()
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
    Bait,
    ShowCaught,
    ShowKilled,
    ShowHarvested,
}

impl Filter {
    fn key(self) -> &'static str {
        match self {
            Self::Location => "dogonline_companion.filter.location",
            Self::Time => "dogonline_companion.filter.time",
            Self::Weather => "dogonline_companion.filter.weather",
            Self::Bait => "dogonline_companion.filter.bait",
            Self::ShowCaught => "dogonline_companion.filter.show_caught",
            Self::ShowKilled => "dogonline_companion.filter.show_killed",
            Self::ShowHarvested => "dogonline_companion.filter.show_harvested",
        }
    }
}

pub fn load_filter_string(filter: Filter) -> String {
    load_string(filter.key()).unwrap_or_default()
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
    load_names(CAUGHT_FISH_KEY)
}

pub fn save_caught_fish(names: &HashSet<String>) {
    save_names(CAUGHT_FISH_KEY, names);
}

pub fn load_killed_enemies() -> HashSet<String> {
    load_names(KILLED_ENEMIES_KEY)
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
    load_names(HARVESTED_ITEMS_KEY)
}

pub fn save_harvested_items(names: &HashSet<String>) {
    save_names(HARVESTED_ITEMS_KEY, names);
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
