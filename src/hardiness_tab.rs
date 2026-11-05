use crate::formatting::image_path;
use crate::hardiness::{HardinessData, ItemType};
use crate::long_press::LongPress;
use crate::progress_counts::{Progress, ProgressCounts};
use crate::storage::{self, Filter};
use leptos::prelude::*;

const SUB_TABS: [(ItemType, &str); 4] = [
    (ItemType::Crop, "Crops"),
    (ItemType::Gem, "Gems"),
    (ItemType::Bug, "Bugs"),
    (ItemType::Egg, "Eggs"),
];

/// Flips `harvested` for the item at `index` and saves the result to browser storage.
fn toggle_harvested(data: RwSignal<HardinessData>, index: usize) {
    data.update(|data| {
        if let Some(item) = data.items.get_mut(index) {
            item.harvested = !item.harvested;
        }
    });

    storage::save_harvested_items(&data.with_untracked(HardinessData::harvested_names));
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn HardinessTab(data: RwSignal<HardinessData>) -> impl IntoView {
    let (selected, set_selected) = signal(ItemType::Crop);
    let (show_harvested, set_show_harvested) =
        signal(storage::load_filter_bool(Filter::ShowHarvested, true));

    Effect::new(move |_| {
        storage::save_filter_bool(Filter::ShowHarvested, show_harvested.get());
    });

    let total_progress = Signal::derive(move || {
        data.with(|data| {
            Progress::of(
                data.items
                    .iter()
                    .map(|item| (item.harvested, item.hardiness)),
            )
        })
    });

    let progress = Signal::derive(move || {
        let selected = selected.get();

        data.with(|data| {
            Progress::of(
                data.items
                    .iter()
                    .filter(|item| item.item_type == selected)
                    .map(|item| (item.harvested, item.hardiness)),
            )
        })
    });

    view! {
        <section class="tab-content">
            <h2>"Hardiness"</h2>

            <p class="hint">"Long-press (or right-click) an item to mark it as harvested"</p>

            <ProgressCounts
                progress=total_progress
                show_done=show_harvested
                items="All items"
                done_label="harvested"
                stat="Total hardiness"
            />

            <nav class="tabs sub-tabs" aria-label="Hardiness categories">
                {SUB_TABS
                    .iter()
                    .map(|(item_type, label)| {
                        let item_type = *item_type;

                        view! {
                            <button
                                class="tab-button"
                                class:active=move || selected.get() == item_type
                                aria-pressed=move || selected.get() == item_type
                                on:click=move |_| set_selected.set(item_type)
                            >
                                {*label}
                            </button>
                        }
                    })
                    .collect_view()}
            </nav>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_harvested.get()
                    on:change=move |ev| {
                        set_show_harvested.set(event_target_checked(&ev));
                    }
                />
                " Show harvested"
            </label>

            <ProgressCounts
                progress=progress
                show_done=show_harvested
                items="Items"
                done_label="harvested"
                stat="Hardiness"
            />

            <div class="item-grid">
                {move || {
                    let selected = selected.get();
                    let show_harvested = show_harvested.get();

                    data.get()
                        .items
                        .into_iter()
                        // Enumerate before filtering so each card keeps its index in the full list.
                        .enumerate()
                        .filter(|(_, item)| {
                            item.item_type == selected && (show_harvested || !item.harvested)
                        })
                        .map(|(index, item)| {
                            let long_press = LongPress::new(move || toggle_harvested(data, index));

                            view! {
                                <div
                                    class="item-card"
                                    on:pointerdown={let lp = long_press.clone(); move |ev| lp.pointer_down(&ev)}
                                    on:pointerup={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:pointerleave={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:pointercancel={let lp = long_press.clone(); move |_| lp.cancel()}
                                    on:contextmenu={let lp = long_press.clone(); move |ev| lp.context_menu(&ev)}
                                >
                                    <div class="item-icon-wrap">
                                        <img
                                            class="item-icon"
                                            class:complete=item.harvested
                                            src=image_path("hardiness", &item.name)
                                            alt=""
                                        />
                                    </div>

                                    <p class="item-name">{item.name.clone()}</p>

                                    <p class="item-value">
                                        {format!("+{} hardiness", item.hardiness)}
                                    </p>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </section>
    }
}
