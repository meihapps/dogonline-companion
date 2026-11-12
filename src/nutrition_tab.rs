use crate::filter_select::FilterSelect;
use crate::formatting::image_path;
use crate::long_press::LongPress;
use crate::nutrition::{Food, NutritionData};
use crate::progress_counts::{Progress, ProgressCounts};
use crate::storage::{self, Filter};
use leptos::prelude::*;

/// An empty `food_type` filter matches every food; any other value only matches foods of
/// that type.
fn matches_type(food: &Food, food_type: &str) -> bool {
    food_type.is_empty()
        || food
            .kind
            .is_some_and(|kind| format!("{kind:?}") == food_type)
}

/// Flips `eaten` for the food at `index` and saves the result to browser storage.
fn toggle_eaten(data: RwSignal<NutritionData>, index: usize) {
    data.update(|data| {
        if let Some(food) = data.foods.get_mut(index) {
            food.eaten = !food.eaten;
        }
    });

    storage::save_eaten_foods(&data.with_untracked(NutritionData::eaten_names));
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn NutritionTab(data: RwSignal<NutritionData>) -> impl IntoView {
    let (selected_type, set_selected_type) =
        signal(storage::load_filter_string(Filter::FoodType, ""));
    let (show_eaten, set_show_eaten) = signal(storage::load_filter_bool(Filter::ShowEaten, true));
    let (show_zero, set_show_zero) =
        signal(storage::load_filter_bool(Filter::ShowZeroNutrition, true));

    // Remember the filters between visits.
    Effect::new(move |_| {
        storage::save_filter_string(Filter::FoodType, &selected_type.get());
        storage::save_filter_bool(Filter::ShowEaten, show_eaten.get());
        storage::save_filter_bool(Filter::ShowZeroNutrition, show_zero.get());
    });

    let progress = Signal::derive(move || {
        let selected_type = selected_type.get();
        let show_zero = show_zero.get();

        data.with(|data| {
            Progress::of(
                data.foods
                    .iter()
                    .filter(|food| {
                        matches_type(food, &selected_type) && (show_zero || food.nutrition > 0)
                    })
                    .map(|food| (food.eaten, food.nutrition)),
            )
        })
    });

    view! {
        <section class="tab-content">
            <h2>"Nutrition"</h2>

            <p class="hint">"Long-press (or right-click) a food to mark it as eaten"</p>

            <div class="filters">
                <FilterSelect
                    label="Type"
                    value=selected_type
                    on_change=set_selected_type
                    all_label="All types"
                    options=vec![
                        ("Dairy", "Dairy"),
                        ("Fish", "Fish"),
                        ("Fruit", "Fruit"),
                        ("Meat", "Meat"),
                        ("Other", "Other"),
                    ]
                />
            </div>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_eaten.get()
                    on:change=move |ev| {
                        set_show_eaten.set(event_target_checked(&ev));
                    }
                />
                " Show eaten"
            </label>

            <label class="show-caught-filter">
                <input
                    type="checkbox"
                    prop:checked=move || show_zero.get()
                    on:change=move |ev| {
                        set_show_zero.set(event_target_checked(&ev));
                    }
                />
                " Show 0 nutrition"
            </label>

            <ProgressCounts
                progress=progress
                show_done=show_eaten
                items="Foods"
                done_label="eaten"
                stat="Nutrition"
            />

            <div class="item-grid">
                {move || {
                    let selected_type = selected_type.get();
                    let show_eaten = show_eaten.get();
                    let show_zero = show_zero.get();

                    data.get()
                        .foods
                        .into_iter()
                        // Enumerate before filtering so each card keeps its index in the full list.
                        .enumerate()
                        .filter(|(_, food)| {
                            matches_type(food, &selected_type)
                                && (show_eaten || !food.eaten)
                                && (show_zero || food.nutrition > 0)
                        })
                        .map(|(index, food)| {
                            let long_press = LongPress::new(move || toggle_eaten(data, index));

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
                                            class:complete=food.eaten
                                            src=image_path("nutrition", &food.name)
                                            alt=""
                                        />
                                    </div>

                                    <p class="item-name">{food.name.clone()}</p>

                                    <p class="item-value">
                                        {format!("+{} nutrition", food.nutrition)}
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
