use crate::storage;
use leptos::prelude::*;
use std::collections::HashSet;
use std::time::Duration;

const DAILIES: [&str; 7] = [
    "Go fishing",
    "Complete a sea dog quest",
    "Tend the farm",
    "Rummage through the bush",
    "Play \"Treat or Get Blown Up\"",
    "Walk up to the cactus",
    "Beat the Temple of Dog",
];

/// How often to check whether the daily reset has passed while the page is open.
const RESET_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// The current calendar date (YYYY-MM-DD) in Pacific time. When this changes, it's past
/// midnight Pacific and the dailies reset.
pub fn reset_day() -> String {
    use web_sys::js_sys::{Date, Object, Reflect};

    let options = Object::new();
    let _ = Reflect::set(&options, &"timeZone".into(), &"America/Los_Angeles".into());

    // The "en-CA" locale formats dates as YYYY-MM-DD.
    Date::new_0()
        .to_locale_date_string("en-CA", &options)
        .into()
}

/// Starts a timer that clears the checklist once midnight Pacific passes (or when another
/// browser tab has changed it). Call once, from `App`, which lives as long as the page.
pub fn start_reset_timer(done: RwSignal<HashSet<String>>) {
    set_interval(
        move || {
            let fresh = storage::load_dailies(&reset_day());

            if fresh != done.get_untracked() {
                done.set(fresh);
            }
        },
        RESET_CHECK_INTERVAL,
    );
}

fn toggle(done: RwSignal<HashSet<String>>, task: &str) {
    done.update(|done| {
        if !done.remove(task) {
            done.insert(task.to_string());
        }
    });

    storage::save_dailies(&reset_day(), &done.get_untracked());
}

#[component]
pub fn DailiesTab(done: RwSignal<HashSet<String>>) -> impl IntoView {
    view! {
        <section class="tab-content">
            <h2>"Dailies"</h2>

            <p class="hint">"Resets at midnight Pacific time"</p>

            <ul class="dailies-list">
                {DAILIES
                    .iter()
                    .map(|task| {
                        let task = *task;

                        view! {
                            <li>
                                <label class="daily-item">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || done.with(|done| done.contains(task))
                                        on:change=move |_| toggle(done, task)
                                    />
                                    <span class:done=move || done.with(|done| done.contains(task))>
                                        {task}
                                    </span>
                                </label>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}
