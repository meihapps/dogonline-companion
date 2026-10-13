use crate::coordination::{Bait, CoordinationData, Fish};
use crate::formatting::{format_enum, format_list};
use crate::storage::{self, Filter};
use leptos::prelude::*;

fn bait_tier(bait: &Bait) -> (u8, u8) {
    match bait {
        // Standard bait progression
        Bait::BucketOfWorms => (0, 0),
        Bait::MoundOfMinnows => (0, 1),
        Bait::CupOfCrickets => (0, 2),
        Bait::PouchOfPrawns => (0, 3),
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
        "GoldenGrub" => Some((0, 4)),
        "BalloonHook" => Some((1, 0)),
        "WickedSkymetalHook" => Some((1, 1)),
        "SkyTackle" => Some((1, 2)),
        _ => None,
    }
}

fn matches_bait(fish: &Fish, selected: &str) -> bool {
    let Some((selected_line, selected_tier)) = selected_bait_tier(selected) else {
        return true;
    };

    let (fish_line, fish_tier) = bait_tier(&fish.bait);

    fish_line == selected_line && fish_tier <= selected_tier
}

fn matches_filters(fish: &Fish, location: &str, time: &str, weather: &str, bait: &str) -> bool {
    let location_matches = location.is_empty()
        || fish
            .location
            .iter()
            .any(|spot| format!("{spot:?}") == location);

    let time_matches = time.is_empty() || fish.time.iter().any(|t| format!("{t:?}") == time);

    let weather_matches =
        weather.is_empty() || fish.weather.iter().any(|w| format!("{w:?}") == weather);

    let bait_matches = bait.is_empty() || matches_bait(fish, bait);

    location_matches && time_matches && weather_matches && bait_matches
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
fn BaitFilter(value: ReadSignal<String>, on_change: WriteSignal<String>) -> impl IntoView {
    view! {
        <label>
            "Bait"

            <select
                prop:value=move || value.get()
                on:change=move |ev| {
                    on_change.set(event_target_value(&ev));
                }
            >
                <option value="" selected=move || value.get().is_empty()>"All bait"</option>

                <optgroup label="Standard Bait">
                    <option value="BucketOfWorms" selected=move || value.get() == "BucketOfWorms">
                        "Bucket of Worms"
                    </option>
                    <option value="MoundOfMinnows" selected=move || value.get() == "MoundOfMinnows">
                        "Mound of Minnows"
                    </option>
                    <option value="CupOfCrickets" selected=move || value.get() == "CupOfCrickets">
                        "Cup of Crickets"
                    </option>
                    <option value="PouchOfPrawns" selected=move || value.get() == "PouchOfPrawns">
                        "Pouch of Prawns"
                    </option>
                    <option value="GoldenGrub" selected=move || value.get() == "GoldenGrub">
                        "Golden Grub"
                    </option>
                </optgroup>

                <optgroup label="Hook Bait">
                    <option value="BalloonHook" selected=move || value.get() == "BalloonHook">
                        "Balloon Hook"
                    </option>
                    <option value="WickedSkymetalHook" selected=move || value.get() == "WickedSkymetalHook">
                        "Wicked Skymetal Hook"
                    </option>
                    <option value="SkyTackle" selected=move || value.get() == "SkyTackle">
                        "Sky Tackle"
                    </option>
                </optgroup>
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
    selected_bait: ReadSignal<String>,
    set_selected_bait: WriteSignal<String>,
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

            <BaitFilter
                value=selected_bait
                on_change=set_selected_bait
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

    view! {
        <button
            class="item-card"
            on:click=move |_| {
                set_selected_fish.set(Some(fish.clone()));
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
                toggle_caught(data, fish_index);
            }
        >
            <div class="item-icon-wrap">
                <div
                    class=if is_caught {
                        "item-icon-placeholder caught"
                    } else {
                        "item-icon-placeholder"
                    }
                ></div>
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

#[component]
fn FishGrid(
    data: RwSignal<CoordinationData>,
    selected_location: ReadSignal<String>,
    selected_time: ReadSignal<String>,
    selected_weather: ReadSignal<String>,
    selected_bait: ReadSignal<String>,
    show_caught: ReadSignal<bool>,
    set_selected_fish: WriteSignal<Option<Fish>>,
) -> impl IntoView {
    view! {
        <div class="item-grid">
            {move || {
                let location = selected_location.get();
                let time = selected_time.get();
                let weather = selected_weather.get();
                let bait = selected_bait.get();
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
                            &bait,
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
                    <div class="modal-image"></div>

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
        signal(storage::load_filter_string(Filter::Location));

    let (selected_time, set_selected_time) = signal(storage::load_filter_string(Filter::Time));

    let (selected_weather, set_selected_weather) =
        signal(storage::load_filter_string(Filter::Weather));

    let (selected_bait, set_selected_bait) = signal(storage::load_filter_string(Filter::Bait));

    let (show_caught, set_show_caught) =
        signal(storage::load_filter_bool(Filter::ShowCaught, true));

    // Remember the filters between visits.
    Effect::new(move |_| {
        storage::save_filter_string(Filter::Location, &selected_location.get());
        storage::save_filter_string(Filter::Time, &selected_time.get());
        storage::save_filter_string(Filter::Weather, &selected_weather.get());
        storage::save_filter_string(Filter::Bait, &selected_bait.get());
        storage::save_filter_bool(Filter::ShowCaught, show_caught.get());
    });

    let close_modal = Callback::new(move |()| {
        set_selected_fish.set(None);
    });

    view! {
        <section class="tab-content">
            <h2>"Coordination"</h2>

            <p class="hint">"Right-click a fish to mark it as caught"</p>

            <CoordinationFilters
                selected_location=selected_location
                set_selected_location=set_selected_location
                selected_time=selected_time
                set_selected_time=set_selected_time
                selected_weather=selected_weather
                set_selected_weather=set_selected_weather
                selected_bait=selected_bait
                set_selected_bait=set_selected_bait
            />

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

            <FishGrid
                data=data
                selected_location=selected_location
                selected_time=selected_time
                selected_weather=selected_weather
                selected_bait=selected_bait
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
