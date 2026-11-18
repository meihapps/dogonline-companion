use crate::coordination::{Bait, Catch, CoordinationData, Fish, TimeOfDay, Weather};
use crate::filter_select::FilterSelect;
use crate::formatting::{format_enum, format_list, image_path};
use crate::long_press::LongPress;
use crate::progress_counts::{Progress, ProgressCounts};
use crate::storage::{self, Filter};
use leptos::prelude::*;

/// A line of baits, worst to best: a better bait in a line also catches the fish of the
/// ones before it.
#[derive(Clone, Copy)]
struct BaitLine {
    label: &'static str,
    filter: Filter,
    baits: &'static [(Bait, &'static str)],
}

const BAIT_LINES: [BaitLine; 4] = [
    BaitLine {
        label: "Standard Bait",
        filter: Filter::StandardBait,
        baits: &[
            (Bait::BucketOfWorms, "Bucket of Worms"),
            (Bait::MoundOfMinnows, "Mound of Minnows"),
            (Bait::CupOfCrickets, "Cup of Crickets"),
            (Bait::PouchOfPrawns, "Pouch of Prawns"),
        ],
    },
    BaitLine {
        label: "Hook Bait",
        filter: Filter::HookBait,
        baits: &[
            (Bait::BalloonHook, "Balloon Hook"),
            (Bait::WickedSkymetalHook, "Wicked Skymetal Hook"),
            (Bait::SkyTackle, "Sky Tackle"),
        ],
    },
    BaitLine {
        label: "Animal Feed",
        filter: Filter::FeedBait,
        baits: &[
            (Bait::PremuimAnimalFeed, "Premuim Animal Feed"),
            (Bait::SuperPremuimAnimalFeed, "Super Premuim Animal Feed"),
            (
                Bait::SuperUltraPremiumAnimalFeed,
                "Super Ultra Premium Animal Feed",
            ),
        ],
    },
    BaitLine {
        label: "Candy Bait",
        filter: Filter::CandyBait,
        baits: &[
            (Bait::CandyCornGrub, "Candy Corn Grub"),
            (Bait::GummyChum, "Gummy Chum"),
            (Bait::SourPowderedHook, "Sour-Powdered Hook"),
        ],
    },
];

/// Baits (and cages) with a pool of fish of their own, which no other bait catches. Each
/// is toggled on its own rather than being part of a line.
const OWN_POOL_BAITS: [(Bait, Filter, &str); 8] = [
    (Bait::GoldenGrub, Filter::GoldenGrub, "Golden Grub"),
    (
        Bait::BucketOfRadioactiveWorms,
        Filter::RadioactiveWorms,
        "Bucket of Radioactive Worms",
    ),
    // Uses up the rest of the day's fishing for one rare fish.
    (Bait::BombasticBait, Filter::BombasticBait, "Bombastic Bait"),
    (Bait::Cage, Filter::Cage, "Cage"),
    (Bait::PumpkinCage, Filter::PumpkinCage, "Pumpkin Cage"),
    (
        Bait::SweetiecakeLure,
        Filter::SweetiecakeLure,
        "Sweetiecake Lure",
    ),
    (Bait::BakersBait, Filter::BakersBait, "Baker's Bait"),
    (
        Bait::DeepseaExplorerPod,
        Filter::DeepseaExplorerPod,
        "Deepsea Explorer Pod",
    ),
];

/// The value a bait is stored and selected as, e.g. `"BucketOfWorms"`.
fn bait_key(bait: Bait) -> String {
    format!("{bait:?}")
}

/// The line a bait is in and its place in that line, or `None` for own-pool baits.
fn bait_tier(bait: Bait) -> Option<(usize, usize)> {
    BAIT_LINES.iter().enumerate().find_map(|(line, bait_line)| {
        bait_line
            .baits
            .iter()
            .position(|(line_bait, _)| *line_bait == bait)
            .map(|tier| (line, tier))
    })
}

fn bait_label(bait: Bait) -> &'static str {
    BAIT_LINES
        .iter()
        .flat_map(|line| line.baits.iter().copied())
        .chain(
            OWN_POOL_BAITS
                .iter()
                .map(|(bait, _, label)| (*bait, *label)),
        )
        .find(|(line_bait, _)| *line_bait == bait)
        .map_or("Unknown", |(_, label)| label)
}

/// The best bait picked in each of `BAIT_LINES` (empty for none), the own-pool baits
/// toggled on, and the most coordination a bait can need (`None` for no limit).
#[derive(Clone, Default)]
struct BaitFilter {
    lines: [String; BAIT_LINES.len()],
    own_pool: Vec<Bait>,
    max_coordination: Option<u32>,
}

fn usable(bait: Bait, max_coordination: Option<u32>) -> bool {
    max_coordination.is_none_or(|max| bait.required_coordination() <= max)
}

/// The bait actually used for `selected` in `line`: the best usable one at or below it.
/// Empty if there isn't one.
fn usable_pick(line: &BaitLine, selected: &str, max_coordination: Option<u32>) -> String {
    let Some(tier) = tier_of(line, selected) else {
        return String::new();
    };

    line.baits[..=tier]
        .iter()
        .rev()
        .find(|(bait, _)| usable(*bait, max_coordination))
        .map_or_else(String::new, |(bait, _)| bait_key(*bait))
}

impl BaitFilter {
    /// Only the parts of the filter for baits in `used`. The others aren't shown, so they
    /// shouldn't hide anything either.
    fn limited_to(&self, used: &[Bait]) -> Self {
        Self {
            lines: std::array::from_fn(|line| {
                if line_is_used(&BAIT_LINES[line], used) {
                    self.lines[line].clone()
                } else {
                    String::new()
                }
            }),
            own_pool: self
                .own_pool
                .iter()
                .copied()
                .filter(|bait| used.contains(bait))
                .collect(),
            max_coordination: self.max_coordination,
        }
    }
}

fn line_is_used(line: &BaitLine, used: &[Bait]) -> bool {
    line.baits.iter().any(|(bait, _)| used.contains(bait))
}

fn at_location(catch: &Catch, location: &str) -> bool {
    location.is_empty() || format!("{:?}", catch.location) == location
}

/// Every bait some fish at `location` (anywhere, if empty) is caught with.
fn baits_used_at(fish: &[Fish], location: &str) -> Vec<Bait> {
    let mut used: Vec<Bait> = fish
        .iter()
        .flat_map(|fish| &fish.catches)
        .filter(|catch| at_location(catch, location))
        .map(|catch| catch.bait)
        .collect();
    used.sort_by_key(|bait| bait_key(*bait));
    used.dedup();
    used
}

/// Whether any of the selected baits can catch a fish caught with `bait`.
/// With nothing selected there is no bait filter, so every bait matches.
fn matches_bait(bait: Bait, baits: &BaitFilter) -> bool {
    if baits.lines.iter().all(String::is_empty) && baits.own_pool.is_empty() {
        return true;
    }

    let Some((fish_line, fish_tier)) = bait_tier(bait) else {
        return baits.own_pool.contains(&bait) && usable(bait, baits.max_coordination);
    };
    let line = &BAIT_LINES[fish_line];
    let Some(picked_tier) = tier_of(line, &baits.lines[fish_line]) else {
        return false;
    };

    // Any bait from the fish's own up to the one picked would do, if it can be used yet.
    line.baits
        .get(fish_tier..=picked_tier)
        .is_some_and(|catching| {
            catching
                .iter()
                .any(|(bait, _)| usable(*bait, baits.max_coordination))
        })
}

/// Where the bait stored as `selected` is in `line`.
fn tier_of(line: &BaitLine, selected: &str) -> Option<usize> {
    line.baits
        .iter()
        .position(|(bait, _)| bait_key(*bait) == selected)
}

/// Whether this one way of catching the fish fits every filter at once.
fn catch_matches(
    catch: &Catch,
    location: &str,
    time: &str,
    weather: &str,
    baits: &BaitFilter,
) -> bool {
    // An unknown time or weather (an empty list) matches any filter, rather than hiding
    // the fish.
    let time_matches = time.is_empty()
        || catch.time.is_empty()
        || catch.time.iter().any(|t| format!("{t:?}") == time);

    let weather_matches = weather.is_empty()
        || catch.weather.is_empty()
        || catch.weather.iter().any(|w| format!("{w:?}") == weather);

    at_location(catch, location)
        && time_matches
        && weather_matches
        && matches_bait(catch.bait, baits)
}

/// A fish matches if any one way of catching it fits all the filters.
/// A fish with no known way to catch it matches any filter but a location, so it isn't
/// counted in every area.
fn matches_filters(
    fish: &Fish,
    location: &str,
    time: &str,
    weather: &str,
    baits: &BaitFilter,
) -> bool {
    if fish.catches.is_empty() {
        return location.is_empty();
    }

    fish.catches
        .iter()
        .any(|catch| catch_matches(catch, location, time, weather, baits))
}

#[component]
fn CoordinationFilters(
    selected_location: ReadSignal<String>,
    set_selected_location: WriteSignal<String>,
    selected_time: ReadSignal<String>,
    set_selected_time: WriteSignal<String>,
    selected_weather: ReadSignal<String>,
    set_selected_weather: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="filters">
            <FilterSelect
                label="Location"
                value=selected_location
                on_change=set_selected_location
                all_label="All locations"
                options=AREAS.to_vec()
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
        </div>
    }
}

type LineSignals = [(ReadSignal<String>, WriteSignal<String>); BAIT_LINES.len()];
type OwnPoolSignals = [(Bait, ReadSignal<bool>, WriteSignal<bool>); OWN_POOL_BAITS.len()];

/// The bait filters, folded away. Only the baits in `used` that can be used with
/// `max_coordination` are shown.
#[component]
fn BaitFilters(
    used: Memo<Vec<Bait>>,
    max_coordination: Signal<Option<u32>>,
    lines: LineSignals,
    own_pool: OwnPoolSignals,
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <details
            class="bait-filters"
            prop:open=move || open.get()
            on:toggle=move |ev| {
                set_open.set(event_target::<web_sys::HtmlDetailsElement>(&ev).open());
            }
        >
            <summary>"Baits"</summary>

            <div class="filters">
                {move || {
                    let max_coordination = max_coordination.get();

                    used.with(|used| {
                        BAIT_LINES
                            .iter()
                            .zip(lines)
                            .filter(|(line, _)| line_is_used(line, used))
                            .filter_map(|(line, (value, set_value))| {
                                let options: Vec<(String, &'static str)> = line
                                    .baits
                                    .iter()
                                    .filter(|(bait, _)| usable(*bait, max_coordination))
                                    .map(|(bait, label)| (bait_key(*bait), *label))
                                    .collect();
                                if options.is_empty() {
                                    return None;
                                }

                                // Shows what's actually used when the pick needs more
                                // coordination, without changing the saved pick.
                                let shown = Signal::derive(move || {
                                    usable_pick(line, &value.get(), max_coordination)
                                });

                                Some(view! {
                                    <FilterSelect
                                        label=line.label
                                        value=shown
                                        on_change=set_value
                                        all_label="None"
                                        options=options
                                    />
                                })
                            })
                            .collect_view()
                    })
                }}
            </div>

            <div class="bait-chips">
                {move || {
                    let max_coordination = max_coordination.get();

                    used.with(|used| {
                        own_pool
                            .into_iter()
                            .filter(|(bait, ..)| {
                                used.contains(bait) && usable(*bait, max_coordination)
                            })
                            .map(|(bait, on, set_on)| {
                                view! {
                                    <button
                                        class="bait-chip"
                                        aria-pressed=move || on.get().to_string()
                                        on:click=move |_| set_on.update(|on| *on = !*on)
                                    >
                                        {bait_label(bait)}
                                    </button>
                                }
                            })
                            .collect_view()
                    })
                }}
            </div>
        </details>
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
                {format!("+{} coordination", format_coordination(fish.coordination))}
            </p>
        </button>
    }
}

const AREAS: [(&str, &str); 7] = [
    ("OldFishingHole", "Old Fishing Hole"),
    ("DesertOasis", "Desert Oasis"),
    ("IceFishingHole", "Ice Fishing Hole"),
    ("OceanFishingCruiser", "Ocean Fishing Cruiser"),
    ("FlyFishingShip", "Fly Fishing Ship"),
    ("DogwoodsFishingHole", "Dogwoods Fishing Hole"),
    ("HauntedShip", "Haunted Ship"),
];

/// Shows how many uncaught fish match the current filters in each area.
/// A fish that lives in several areas is counted in each of them.
#[component]
fn UncaughtCounts(
    data: RwSignal<CoordinationData>,
    selected_location: ReadSignal<String>,
    selected_time: ReadSignal<String>,
    selected_weather: ReadSignal<String>,
    baits: Signal<BaitFilter>,
) -> impl IntoView {
    view! {
        <div class="uncaught-section">
            <p class="uncaught-title">"Uncaught fish per area (with current filters)"</p>

            <div class="counts">
            {move || {
                let location = selected_location.get();
                let time = selected_time.get();
                let weather = selected_weather.get();
                let baits = baits.get();

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
                                        && matches_filters(fish, key, &time, &weather, &baits)
                                })
                                .count();

                            view! {
                                <div class="count">
                                    <span class="count-label">{*label}</span>
                                    <span class="count-value">{count}</span>
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
    baits: Signal<BaitFilter>,
    show_caught: ReadSignal<bool>,
    set_selected_fish: WriteSignal<Option<Fish>>,
) -> impl IntoView {
    view! {
        <div class="item-grid">
            {move || {
                let location = selected_location.get();
                let time = selected_time.get();
                let weather = selected_weather.get();
                let baits = baits.get();
                let show_caught = show_caught.get();

                data.get()
                    .fish
                    .into_iter()
                    // Enumerate before filtering so each card keeps its index in the full list.
                    .enumerate()
                    .filter(|(_, fish)| {
                        matches_filters(fish, &location, &time, &weather, &baits)
                            && (show_caught || !fish.caught)
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

fn format_coordination(coordination: Option<u8>) -> String {
    coordination.map_or_else(|| "?".to_string(), |c| c.to_string())
}

/// `items`, or `all` if it's every value and `unknown` if it's none.
fn describe_list<T: std::fmt::Debug>(
    items: &[T],
    every: usize,
    all: &str,
    unknown: &str,
) -> String {
    match items.len() {
        0 => unknown.to_string(),
        len if len == every => all.to_string(),
        _ => format_list(items),
    }
}

const TIMES_OF_DAY: usize = 4;
const WEATHERS: usize = 4;

/// One way to catch a fish, for the fish modal.
#[component]
fn CatchDetails(catch: Catch) -> impl IntoView {
    let time = describe_list::<TimeOfDay>(&catch.time, TIMES_OF_DAY, "Any time", "Time unknown");
    let weather =
        describe_list::<Weather>(&catch.weather, WEATHERS, "Any weather", "Weather unknown");

    view! {
        <li class="catch">
            <p>
                <strong>{bait_label(catch.bait)}</strong>
                " at "
                {format_enum(&catch.location)}
            </p>
            <p class="catch-when">{format!("{time} · {weather}")}</p>
        </li>
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
                            {format!("+{}", format_coordination(fish.coordination))}
                        </p>

                        // Every way to catch it, whatever the filters.
                        {if fish.catches.is_empty() {
                            view! {
                                <p>
                                    <strong>"How to catch: "</strong>
                                    "Unknown"
                                </p>
                            }
                            .into_any()
                        } else {
                            view! {
                                <ul class="catches">
                                    {fish
                                        .catches
                                        .iter()
                                        .cloned()
                                        .map(|catch| view! { <CatchDetails catch=catch /> })
                                        .collect_view()}
                                </ul>
                            }
                            .into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn CoordinationTab(data: RwSignal<CoordinationData>) -> impl IntoView {
    let (selected_fish, set_selected_fish) = signal(None::<Fish>);

    let (selected_location, set_selected_location) =
        signal(storage::load_filter_string(Filter::Location, ""));

    let (selected_time, set_selected_time) = signal(storage::load_filter_string(Filter::Time, ""));

    let (selected_weather, set_selected_weather) =
        signal(storage::load_filter_string(Filter::Weather, ""));

    // Default to the best baits, which between them catch every fish.
    let lines: LineSignals = BAIT_LINES.map(|line| {
        let best = line
            .baits
            .last()
            .map_or_else(String::new, |(bait, _)| bait_key(*bait));
        signal(storage::load_filter_string(line.filter, &best))
    });

    let own_pool: OwnPoolSignals = OWN_POOL_BAITS.map(|(bait, filter, _)| {
        let (on, set_on) = signal(storage::load_filter_bool(filter, true));
        (bait, on, set_on)
    });

    let (baits_by_location, set_baits_by_location) =
        signal(storage::load_filter_bool(Filter::BaitsByLocation, true));

    // Off by default: with no fish marked caught, it would hide almost every bait.
    let (baits_by_coordination, set_baits_by_coordination) = signal(storage::load_filter_bool(
        Filter::BaitsByCoordination,
        false,
    ));

    // The coordination earned from caught fish.
    let coordination = Memo::new(move |_| {
        data.with(|data| {
            data.fish
                .iter()
                .filter(|fish| fish.caught)
                .map(|fish| u32::from(fish.coordination.unwrap_or(0)))
                .sum::<u32>()
        })
    });

    let max_coordination =
        Signal::derive(move || baits_by_coordination.get().then(|| coordination.get()));

    let used = Memo::new(move |_| {
        let location = if baits_by_location.get() {
            selected_location.get()
        } else {
            String::new()
        };
        data.with(|data| baits_used_at(&data.fish, &location))
    });

    let picked_baits = Signal::derive(move || BaitFilter {
        lines: lines.map(|(value, _)| value.get()),
        own_pool: own_pool
            .iter()
            .filter(|(_, on, _)| on.get())
            .map(|(bait, ..)| *bait)
            .collect(),
        max_coordination: max_coordination.get(),
    });

    let baits = Signal::derive(move || used.with(|used| picked_baits.get().limited_to(used)));

    let (baits_open, set_baits_open) = signal(storage::load_filter_bool(Filter::BaitsOpen, false));

    let (show_caught, set_show_caught) =
        signal(storage::load_filter_bool(Filter::ShowCaught, true));

    // Remember the filters between visits.
    Effect::new(move |_| {
        storage::save_filter_string(Filter::Location, &selected_location.get());
        storage::save_filter_string(Filter::Time, &selected_time.get());
        storage::save_filter_string(Filter::Weather, &selected_weather.get());
        for (line, (value, _)) in BAIT_LINES.iter().zip(lines) {
            storage::save_filter_string(line.filter, &value.get());
        }
        for ((_, filter, _), (_, on, _)) in OWN_POOL_BAITS.iter().zip(own_pool) {
            storage::save_filter_bool(*filter, on.get());
        }
        storage::save_filter_bool(Filter::BaitsOpen, baits_open.get());
        storage::save_filter_bool(Filter::BaitsByLocation, baits_by_location.get());
        storage::save_filter_bool(Filter::BaitsByCoordination, baits_by_coordination.get());
        storage::save_filter_bool(Filter::ShowCaught, show_caught.get());
    });

    let progress = Signal::derive(move || {
        let location = selected_location.get();
        let time = selected_time.get();
        let weather = selected_weather.get();
        let baits = baits.get();

        data.with(|data| {
            Progress::of(
                data.fish
                    .iter()
                    .filter(|fish| matches_filters(fish, &location, &time, &weather, &baits))
                    .map(|fish| (fish.caught, fish.coordination.unwrap_or(0))),
            )
        })
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
            />

            <BaitFilters
                used=used
                max_coordination=max_coordination
                lines=lines
                own_pool=own_pool
                open=baits_open
                set_open=set_baits_open
            />

            <div class="checkbox-filters">
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

                <label class="show-caught-filter">
                    <input
                        type="checkbox"
                        prop:checked=move || baits_by_location.get()
                        on:change=move |ev| {
                            set_baits_by_location.set(event_target_checked(&ev));
                        }
                    />
                    " Filter baits by location"
                </label>

                <label class="show-caught-filter">
                    <input
                        type="checkbox"
                        prop:checked=move || baits_by_coordination.get()
                        on:change=move |ev| {
                            set_baits_by_coordination.set(event_target_checked(&ev));
                        }
                    />
                    " Filter baits by coordination"
                </label>
            </div>

            <ProgressCounts
                progress=progress
                show_done=show_caught
                items="Fish"
                done_label="caught"
                stat="Coordination"
            />

            <UncaughtCounts
                data=data
                selected_location=selected_location
                selected_time=selected_time
                selected_weather=selected_weather
                baits=baits
            />

            <FishGrid
                data=data
                selected_location=selected_location
                selected_time=selected_time
                selected_weather=selected_weather
                baits=baits
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
    use crate::coordination::FishingSpot;

    fn catch(location: FishingSpot, bait: Bait, time: &[TimeOfDay], weather: &[Weather]) -> Catch {
        Catch {
            location,
            bait,
            time: time.to_vec(),
            weather: weather.to_vec(),
        }
    }

    fn fish(catches: Vec<Catch>) -> Fish {
        Fish {
            name: "Test Fish".to_string(),
            catches,
            coordination: Some(1),
            caught: false,
        }
    }

    fn baits(standard: &str, hook: &str, feed: &str, own_pool: &[Bait]) -> BaitFilter {
        BaitFilter {
            lines: [standard, hook, feed, ""].map(str::to_string),
            own_pool: own_pool.to_vec(),
            max_coordination: None,
        }
    }

    #[test]
    fn better_bait_also_catches_lower_tier_fish() {
        let bait = Bait::MoundOfMinnows;

        assert!(matches_bait(bait, &baits("CupOfCrickets", "", "", &[])));
        assert!(!matches_bait(bait, &baits("BucketOfWorms", "", "", &[])));
    }

    #[test]
    fn better_candy_bait_also_catches_lower_tier_fish() {
        let candy = |selected: &str| BaitFilter {
            lines: ["", "", "", selected].map(str::to_string),
            ..BaitFilter::default()
        };

        assert!(matches_bait(
            Bait::CandyCornGrub,
            &candy("SourPowderedHook")
        ));
        assert!(!matches_bait(Bait::SourPowderedHook, &candy("GummyChum")));
    }

    #[test]
    fn own_pool_baits_only_catch_their_own_fish() {
        let golden_grub = baits("", "", "", &[Bait::GoldenGrub]);

        assert!(!matches_bait(Bait::BucketOfWorms, &golden_grub));
        assert!(matches_bait(Bait::GoldenGrub, &golden_grub));
        assert!(!matches_bait(Bait::Cage, &golden_grub));
    }

    #[test]
    fn standard_bait_does_not_catch_own_pool_fish() {
        let standard = baits("PouchOfPrawns", "", "", &[]);

        assert!(!matches_bait(Bait::GoldenGrub, &standard));
        assert!(!matches_bait(Bait::BucketOfRadioactiveWorms, &standard));
    }

    #[test]
    fn bait_lines_are_separate() {
        assert!(!matches_bait(
            Bait::BucketOfWorms,
            &baits("", "SkyTackle", "", &[])
        ));
        assert!(!matches_bait(
            Bait::BalloonHook,
            &baits("PouchOfPrawns", "", "", &[])
        ));
    }

    #[test]
    fn any_selected_bait_can_catch_the_fish() {
        let filter = baits("BucketOfWorms", "BalloonHook", "", &[Bait::GoldenGrub]);
        let selected = |bait| matches_bait(bait, &filter);

        assert!(selected(Bait::BucketOfWorms));
        assert!(selected(Bait::BalloonHook));
        assert!(selected(Bait::GoldenGrub));
        assert!(!selected(Bait::MoundOfMinnows));
        assert!(!selected(Bait::SkyTackle));
    }

    #[test]
    fn fish_with_no_known_catch_matches_anything_but_a_location() {
        let fish = fish(Vec::new());
        let any = |baits: &BaitFilter| matches_filters(&fish, "", "Night", "Rainy", baits);

        assert!(any(&baits("BucketOfWorms", "", "", &[])));
        assert!(any(&baits("", "", "", &[Bait::GoldenGrub])));
        assert!(!matches_filters(
            &fish,
            "OldFishingHole",
            "",
            "",
            &BaitFilter::default()
        ));
    }

    #[test]
    fn one_catch_has_to_fit_every_filter() {
        // Like the Clam: a Cage at night in the rain, or Cup of Crickets at night.
        let clam = fish(vec![
            catch(
                FishingSpot::OceanFishingCruiser,
                Bait::Cage,
                &[TimeOfDay::Night],
                &[Weather::Rainy],
            ),
            catch(
                FishingSpot::OceanFishingCruiser,
                Bait::CupOfCrickets,
                &[TimeOfDay::Night],
                &[],
            ),
        ]);
        let cage_only = baits("", "", "", &[Bait::Cage]);
        let at = |time, weather, baits: &BaitFilter| {
            matches_filters(&clam, "OceanFishingCruiser", time, weather, baits)
        };

        assert!(at("Night", "Rainy", &cage_only));
        assert!(!at("Night", "Snowy", &cage_only));
        // Cup of Crickets' weather isn't known, so it isn't ruled out.
        assert!(at("Night", "Snowy", &baits("CupOfCrickets", "", "", &[])));
        assert!(!at("Morning", "", &BaitFilter::default()));
        assert!(!matches_filters(
            &clam,
            "OldFishingHole",
            "",
            "",
            &BaitFilter::default()
        ));
    }

    #[test]
    fn baits_used_at_a_location_only_count_catches_there() {
        let fish = [fish(vec![
            catch(FishingSpot::OldFishingHole, Bait::Cage, &[], &[]),
            catch(FishingSpot::DesertOasis, Bait::GoldenGrub, &[], &[]),
        ])];

        assert_eq!(baits_used_at(&fish, "DesertOasis"), [Bait::GoldenGrub]);
        assert_eq!(baits_used_at(&fish, "").len(), 2);
    }

    #[test]
    fn no_bait_selected_matches_every_fish() {
        let none = BaitFilter::default();

        assert!(matches_bait(Bait::GoldenGrub, &none));
        assert!(matches_bait(Bait::SkyTackle, &none));
    }

    #[test]
    fn baits_not_used_at_the_location_dont_filter() {
        // A standard bait is picked, but nothing here is caught with one.
        let filter = baits("BucketOfWorms", "", "", &[]).limited_to(&[Bait::CandyCornGrub]);

        assert!(matches_bait(Bait::CandyCornGrub, &filter));
    }

    #[test]
    fn baits_needing_more_coordination_cant_catch_anything() {
        let filter = BaitFilter {
            max_coordination: Some(5),
            ..baits("PouchOfPrawns", "", "", &[Bait::GoldenGrub])
        };

        // Mound of Minnows needs 3, so it still catches its fish.
        assert!(matches_bait(Bait::MoundOfMinnows, &filter));
        // Cup of Crickets needs 10, and Pouch of Prawns 25.
        assert!(!matches_bait(Bait::CupOfCrickets, &filter));
        assert!(!matches_bait(Bait::GoldenGrub, &filter));
    }

    #[test]
    fn usable_pick_falls_back_to_the_best_usable_bait() {
        let standard = &BAIT_LINES[0];

        assert_eq!(
            usable_pick(standard, "PouchOfPrawns", Some(12)),
            "CupOfCrickets"
        );
        assert_eq!(
            usable_pick(standard, "PouchOfPrawns", None),
            "PouchOfPrawns"
        );
        assert_eq!(usable_pick(standard, "", Some(12)), "");
    }

    #[test]
    fn every_bait_has_a_label() {
        // Otherwise a new bait would be treated as one with its own pool.
        let data: CoordinationData = serde_json::from_str(crate::COORDINATION_DATA).unwrap();

        for catch in data.fish.iter().flat_map(|fish| &fish.catches) {
            assert_ne!(bait_label(catch.bait), "Unknown", "{:?}", catch.bait);
        }
    }
}
