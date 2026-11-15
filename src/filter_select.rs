use leptos::prelude::*;

/// A dropdown's options as (value, label) pairs. Values can be built at runtime, labels
/// can't.
pub struct FilterOptions(Vec<(String, &'static str)>);

impl From<Vec<(&'static str, &'static str)>> for FilterOptions {
    fn from(options: Vec<(&'static str, &'static str)>) -> Self {
        Self(
            options
                .into_iter()
                .map(|(value, label)| (value.to_string(), label))
                .collect(),
        )
    }
}

impl From<Vec<(String, &'static str)>> for FilterOptions {
    fn from(options: Vec<(String, &'static str)>) -> Self {
        Self(options)
    }
}

/// A labelled dropdown whose empty value means "no filter" (shown as `all_label`).
#[component]
pub fn FilterSelect(
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    on_change: WriteSignal<String>,
    #[prop(into)] options: FilterOptions,
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
                    .0
                    .into_iter()
                    .map(|(option_value, option_label)| {
                        let selected = option_value.clone();

                        view! {
                            <option
                                value=option_value
                                selected=move || value.get() == selected
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
