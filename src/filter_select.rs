use leptos::prelude::*;

/// A labelled dropdown whose empty value means "no filter" (shown as `all_label`).
#[component]
pub fn FilterSelect(
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
