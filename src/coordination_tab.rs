use crate::coordination::{Bait, CoordinationData, Fish};
use crate::formatting::{format_enum, format_list, image_path};
use crate::long_press::LongPress;
use crate::storage::{self, Filter};
use leptos::prelude::*;

fn bait_tier(bait: &Bait) -> (u8, u8) {
    match bait {
        // Standard bait progression
        Bait::BucketOfWorms => (0, 0),
        Bait::MoundOfMinnows => (0, 1),
        Bait::CupOfCrickets => (0, 2),
        Bait::PouchOfPrawns => (0, 3),
        // Golden Grub has its own loot pool: no other standard bait reaches this tier.
        Bait::GoldenGrub => (0, 4),

        // Hook bait progression
        Bait::BalloonHook => (1, 0),
        Bait::WickedSkymetalHook => (1, 1),
        Bait::SkyTackle => (1, 2),
    }
}

fn selected_bait_tier(value: &str) -> Option<(u8, u8)> {
    match value {
        "BucketOfWorms" => Some((0, 0)),
        "MoundOfMinnows" => Some((0, 1)),
        "CupOfCrickets" => Some((0, 2)),
        "PouchOfPrawns" => Some((0, 3)),
        "BalloonHook" => Some((1, 0)),
        "WickedSkymetalHook" => Some((1, 1)),
        "SkyTackle" => Some((1, 2)),
        _ => None,
    }
}

/// A fish matches if any of the selected baits can catch it.
/// With nothing selected there is no bait filter, so every fish matches.
fn matches_bait(fish: &Fish, standard: &str, hook: &str, golden_grub: bool) -> bool {
    if standard.is_empty() && hook.is_empty() && !golden_grub {
        return true;
    }

    let (fish_line, fish_tier) = bait_tier(&fish.bait);

    let catches = |selected: &str| {
        selected_bait_tier(selected)
            .is_some_and(|(line, tier)| line == fish_line && fish_tier <= tier)
    };

    catches(standard) || catches(hook) || (golden_grub && matches!(fish.bait, Bait::GoldenGrub))
}

fn matches_filters(
    fish: &Fish,
    location: &str,
    time: &str,
    weather: &str,
    standard_bait: &str,
    hook_bait: &str,
    golden_grub: bool,
) -> bool {
    let location_matches = location.is_empty()
        || fish
            .location
            .iter()
            .any(|spot| format!("{spot:?}") == location);

    let time_matches = time.is_empty() || fish.time.iter().any(|t| format!("{t:?}") == time);

    let weather_matches =
        weather.is_empty() || fish.weather.iter().any(|w| format!("{w:?}") == weather);

    location_matches
        && time_matches
        && weather_matches
        && matches_bait(fish, standard_bait, hook_bait, golden_grub)
}

#[component]
fn FilterSelect(
    label: &'static str,
    value: ReadSignal<String>,
    on_change: WriteSignal<String>,
    options: Vec<(&'static str, &'static str)>,
    all_label: &'static str,
) -> impl IntoView {
    view! {
        <label>
            {label}

            <select
                prop:value=move || value.get()
                on:change=move |ev| {
                    on_change.set(event_target_value(&ev));
                }
            >
                <option value="" selected=move || value.get().is_empty()>{all_label}</option>

                {options
                    .into_iter()
                    .map(|(option_value, option_label)| {
                        view! {
                            <option
                                value=option_value
                                selected=move || value.get() == option_value
                            >
                                {option_label}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}

#[component]
fn CoordinationFilters(
    selected_location: ReadSignal<String>,
    set_selected_location: WriteSignal<String>,
    selected_time: ReadSignal<String>,
    set_selected_time: WriteSignal<String>,
    selected_weather: ReadSignal<String>,
    set_selected_weather: WriteSignal<String>,
    selected_standard_bait: ReadSignal<String>,
    set_selected_standard_bait: WriteSignal<String>,
    selected_hook_bait: ReadSignal<String>,
    set_selected_hook_bait: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="coordination-filters">
            <FilterSelect
                label="Location"
                value=selected_location
                on_change=set_selected_location
                all_label="All locations"
                options=vec![
                    ("OldFishingHole", "Old Fishing Hole"),
                    ("DesertOasis", "Desert Oasis"),
                    ("IceFishingHole", "Ice Fishing Hole"),
                    ("OceanFishingCruiser", "Ocean Fishing Cruiser"),
                    ("FlyFishingShip", "Fly Fishing Ship"),
                ]
            />

            <FilterSelect
                label="Time"
                value=selected_time
                on_change=set_selected_time
                all_label="All times"
                options=vec![
                    ("Morning", "Morning"),
                    ("Afternoon", "Afternoon"),
                    ("Evening", "Evening"),
                    ("Night", "Night"),
                ]
            />

            <FilterSelect
                label="Weather"
                value=selected_weather
                on_change=set_selected_weather
                all_label="All weather"
                options=vec![
                    ("ClearSkies", "Clear Skies"),
                    ("Rainy", "Rainy"),
                    ("Stormy", "Stormy"),
                    ("Snowy", "Snowy"),
                ]
            />

            <FilterSelect
                label="Standard Bait"
                value=selected_standard_bait
                on_change=set_selected_standard_bait
                all_label="None"
                options=vec![
                    ("BucketOfWorms", "Bucket of Worms"),
                    ("MoundOfMinnows", "Mound of Minnows"),
                    ("CupOfCrickets", "Cup of Crickets"),
                    ("PouchOfPrawns", "Pouch of Prawns"),
                ]
            />

            <FilterSelect
                label="Hook Bait"
                value=selected_hook_bait
                on_change=set_selected_hook_bait
                all_label="None"
                options=vec![
                    ("BalloonHook", "Balloon Hook"),
                    ("WickedSkymetalHook", "Wicked Skymetal Hook"),
                    ("SkyTackle", "Sky Tackle"),
                ]
            />
        </div>
    }
}

/// Flips `caught` for the fish at `index` and saves the result to browser storage.
fn toggle_caught(data: RwSignal<CoordinationData>, index: usize) {
    data.update(|data| {
        if let Some(fish) = data.fish.get_mut(index) {
            fish.caught = !fish.caught;
        }
    });

    storage::save_caught_fish(&data.with_untracked(CoordinationData::caught_names));
}

#[component]
fn FishCard(
    fish: Fish,
    fish_index: usize,
    data: RwSignal<CoordinationData>,
    set_selected_fish: WriteSignal<Option<Fish>>,
) -> impl IntoView {
    let is_caught = fish.caught;
    let long_press = LongPress::new(move || toggle_caught(data, fish_index));

    view! {
        <button
            class="item-card"
            on:click=move |_| {
                if LongPress::take_fired() {
                    return;
                }
                set_selected_fish.set(Some(fish.clone()));
            }
            on:pointerdown={let lp = long_press.clone(); move |ev| lp.pointer_down(&ev)}
            on:pointerup={let lp = long_press.clone(); move |_| lp.cancel()}
            on:pointerleave={let lp = long_press.clone(); move |_| lp.cancel()}
            on:pointercancel={let lp = long_press.clone(); move |_| lp.cancel()}
            on:contextmenu={let lp = long_press.clone(); move |ev| lp.context_menu(&ev)}
        >
            <div class="item-icon-wrap">
                <img
                    class="item-icon"
                    class:complete=is_caught
                    src=image_path("coordination", &fish.name)
                    alt=""
                />
            </div>

            <p class="item-name">
                {fish.name.clone()}
            </p>

            <p class="item-value">
                {format!("+{} coordination", fish.coordination)}
            </p>
        </button>
    }
}

const AREAS: [(&str, &str); 5] = [
    ("OldFishingHole", "Old Fishing Hole"),
    ("DesertOasis", "Desert Oasis"),
    ("IceFishingHole", "Ice Fishing Hole"),
    ("OceanFishingCruiser", "Ocean Fishing Cruiser"),
    ("FlyFishingShip", "Fly Fishing Ship"),
];

/// Shows how many uncaught fish match the current filters in each area.
/// A fish that lives in several areas is counted in each of them.
#[component]
fn UncaughtCounts(
    data: RwSignal<CoordinationData>,
    selected_location: ReadSignal<String>,
    selected_time: ReadSignal<String>,
    selected_weather: ReadSignal<String>,
    selected_standard_bait: ReadSignal<String>,
    selected_hook_bait: ReadSignal<String>,
    golden_grub: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="uncaught-section">
            <p class="uncaught-title">"Uncaught fish per area (with current filters)"</p>

            <div class="uncaught-counts">
            {move || {
                let location = selected_location.get();
                let time = selected_time.get();
                let weather = selected_weather.get();
                let standard_bait = selected_standard_bait.get();
                let hook_bait = selected_hook_bait.get();
                let golden_grub = golden_grub.get();

                data.with(|data| {
                    AREAS
                        .iter()
                        // With an area filter set, only that area is shown.
                        .filter(|(key, _)| location.is_empty() || location == *key)
                        .map(|(key, label)| {
                            let count = data
                                .fish
                                .iter()
                                .filter(|fish| {
                                    !fish.caught
                                        && matches_filters(
                                            fish,
                                            key,
                                            &time,
                                            &weather,
                                            &standard_bait,
                                            &hook_bait,
                                            golden_grub,
                                        )
                                })
                                .count();

                            view! {
                                <div class="uncaught-count">
                                    <span class="uncaught-count-label">{*label}</span>
                                    <span class="uncaught-count-value">{count}</span>
                                </div>
                            }
                        })
                        .collect_view()
                })
            }}
            </div>
        </div>
    }
}

#[component]
fn FishGrid(
    data: RwSignal<CoordinationData>,
    selected_location: ReadSignal<String>,
    selected_time: ReadSignal<String>,
    selected_weather: ReadSignal<String>,
    selected_standard_bait: ReadSignal<String>,
    selected_hook_bait: ReadSignal<String>,
    golden_grub: ReadSignal<bool>,
    show_caught: ReadSignal<bool>,
    set_selected_fish: WriteSignal<Option<Fish>>,
) -> impl IntoView {
    view! {
        <div class="item-grid">
            {move || {
                let location = selected_location.get();
                let time = selected_time.get();
                let weather = selected_weather.get();
                let standard_bait = selected_standard_bait.get();
                let hook_bait = selected_hook_bait.get();
                let golden_grub = golden_grub.get();
                let show_caught = show_caught.get();

                data.get()
                    .fish
                    .into_iter()
                    // Enumerate before filtering so each card keeps its index in the full list.
                    .enumerate()
                    .filter(|(_, fish)| {
                        matches_filters(
                            fish,
                            &location,
                            &time,
                            &weather,
                            &standard_bait,
                            &hook_bait,
                            golden_grub,
                        ) && (show_caught || !fish.caught)
                    })
                    .map(|(fish_index, fish)| {
                        view! {
                            <FishCard
                                fish=fish
                                fish_index=fish_index
                                data=data
                                set_selected_fish=set_selected_fish
                            />
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

#[component]
fn FishModal(fish: Fish, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div
            class="modal-backdrop"
            on:click=move |_| {
                on_close.run(());
            }
        >
            <div
                class="enemy-modal"
                on:click=|ev| ev.stop_propagation()
            >
                <button
                    class="modal-close"
                    aria-label="Close"
                    on:click=move |_| {
                        on_close.run(());
                    }
                >
                    "×"
                </button>

                <div class="modal-body">
                    <img
                        class="modal-image"
                        class:complete=fish.caught
                        src=image_path("coordination", &fish.name)
                        alt=""
                    />

                    <h2>{fish.name.clone()}</h2>

                    <div class="enemy-stats">
                        <p>
                            <strong>"Coordination: "</strong>
                            {format!("+{}", fish.coordination)}
                        </p>

                        <p>
                            <strong>"Location: "</strong>
                            {format_list(&fish.location)}
                        </p>

                        <p>
                            <strong>"Time: "</strong>
                            {format_list(&fish.time)}
                        </p>

                        <p>
                            <strong>"Weather: "</strong>
                            {format_list(&fish.weather)}
                        </p>

                        <p>
                            <strong>"Bait: "</strong>
                            {format_enum(&fish.bait)}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CoordinationTab(data: RwSignal<CoordinationData>) -> impl IntoView {
    let (selected_fish, set_selected_fish) = signal(None::<Fish>);

    let (selected_location, set_selected_location) =
        signal(storage::load_filter_string(Filter::Location, ""));

    let (selected_time, set_selected_time) = signal(storage::load_filter_string(Filter::Time, ""));

    let (selected_weather, set_selected_weather) =
        signal(storage::load_filter_string(Filter::Weather, ""));

    // Default to the best baits, which between them catch every fish.
    let (selected_standard_bait, set_selected_standard_bait) = signal(storage::load_filter_string(
        Filter::StandardBait,
        "PouchOfPrawns",
    ));

    let (selected_hook_bait, set_selected_hook_bait) =
        signal(storage::load_filter_string(Filter::HookBait, "SkyTackle"));

    let (golden_grub, set_golden_grub) =
        signal(storage::load_filter_bool(Filter::GoldenGrub, true));

    let (show_caught, set_show_caught) =
        signal(storage::load_filter_bool(Filter::ShowCaught, true));

    // Remember the filters between visits.
    Effect::new(move |_| {
        storage::save_filter_string(Filter::Location, &selected_location.get());
        storage::save_filter_string(Filter::Time, &selected_time.get());
        storage::save_filter_string(Filter::Weather, &selected_weather.get());
        storage::save_filter_string(Filter::StandardBait, &selected_standard_bait.get());
        storage::save_filter_string(Filter::HookBait, &selected_hook_bait.get());
        storage::save_filter_bool(Filter::GoldenGrub, golden_grub.get());
        storage::save_filter_bool(Filter::ShowCaught, show_caught.get());
    });

    let close_modal = Callback::new(move |()| {
        set_selected_fish.set(None);
    });

    view! {
        <section class="tab-content">
            <h2>"Coordination"</h2>

            <p class="hint">"Long-press (or right-click) a fish to mark it as caught"</p>

            <CoordinationFilters
                selected_location=selected_location
                set_selected_location=set_selected_location
                selected_time=selected_time
                set_selected_time=set_selected_time
                selected_weather=selected_weather
                set_selected_weather=set_selected_weather
                selected_standard_bait=selected_standard_bait
                set_selected_standard_bait=set_selected_standard_bait
                selected_hook_bait=selected_hook_bait
                set_selected_hook_bait=set_selected_hook_bait
            />

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || golden_grub.get()
                    on:change=move |ev| {
                        set_golden_grub.set(event_target_checked(&ev));
                    }
                />
                " Golden Grub"
            </label>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_caught.get()
                    on:change=move |ev| {
                        set_show_caught.set(event_target_checked(&ev));
                    }
                />
                " Show caught"
            </label>

            <UncaughtCounts
                data=data
                selected_location=selected_location
                selected_time=selected_time
                selected_weather=selected_weather
                selected_standard_bait=selected_standard_bait
                selected_hook_bait=selected_hook_bait
                golden_grub=golden_grub
            />

            <FishGrid
                data=data
                selected_location=selected_location
                selected_time=selected_time
                selected_weather=selected_weather
                selected_standard_bait=selected_standard_bait
                selected_hook_bait=selected_hook_bait
                golden_grub=golden_grub
                show_caught=show_caught
                set_selected_fish=set_selected_fish
            />

            {move || {
                selected_fish.get().map(|fish| {
                    view! {
                        <FishModal
                            fish=fish
                            on_close=close_modal
                        />
                    }
                })
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fish_with_bait(bait: Bait) -> Fish {
        Fish {
            name: "Test Fish".to_string(),
            location: Vec::new(),
            time: Vec::new(),
            weather: Vec::new(),
            bait,
            coordination: 1,
            caught: false,
        }
    }

    #[test]
    fn better_bait_also_catches_lower_tier_fish() {
        let fish = fish_with_bait(Bait::MoundOfMinnows);

        assert!(matches_bait(&fish, "CupOfCrickets", "", false));
        assert!(!matches_bait(&fish, "BucketOfWorms", "", false));
    }

    #[test]
    fn golden_grub_only_catches_its_own_fish() {
        assert!(!matches_bait(
            &fish_with_bait(Bait::BucketOfWorms),
            "",
            "",
            true
        ));
        assert!(matches_bait(
            &fish_with_bait(Bait::GoldenGrub),
            "",
            "",
            true
        ));
    }

    #[test]
    fn standard_bait_does_not_catch_golden_grub_fish() {
        assert!(!matches_bait(
            &fish_with_bait(Bait::GoldenGrub),
            "PouchOfPrawns",
            "",
            false
        ));
    }

    #[test]
    fn bait_lines_are_separate() {
        assert!(!matches_bait(
            &fish_with_bait(Bait::BucketOfWorms),
            "",
            "SkyTackle",
            false
        ));
        assert!(!matches_bait(
            &fish_with_bait(Bait::BalloonHook),
            "PouchOfPrawns",
            "",
            false
        ));
    }

    #[test]
    fn any_selected_bait_can_catch_the_fish() {
        let selected =
            |fish| matches_bait(&fish_with_bait(fish), "BucketOfWorms", "BalloonHook", true);

        assert!(selected(Bait::BucketOfWorms));
        assert!(selected(Bait::BalloonHook));
        assert!(selected(Bait::GoldenGrub));
        assert!(!selected(Bait::MoundOfMinnows));
        assert!(!selected(Bait::SkyTackle));
    }

    #[test]
    fn no_bait_selected_matches_every_fish() {
        assert!(matches_bait(
            &fish_with_bait(Bait::GoldenGrub),
            "",
            "",
            false
        ));
        assert!(matches_bait(
            &fish_with_bait(Bait::SkyTackle),
            "",
            "",
            false
        ));
    }
}
