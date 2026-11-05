use leptos::prelude::*;

/// Totals over the entries that match a tab's current filters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub done_points: u32,
    pub total_points: u32,
}

impl Progress {
    /// Adds up `(done, stat points)` for each entry.
    pub fn of(entries: impl Iterator<Item = (bool, u8)>) -> Self {
        entries.fold(Self::default(), |mut progress, (done, points)| {
            progress.total += 1;
            progress.total_points += u32::from(points);

            if done {
                progress.done += 1;
                progress.done_points += u32::from(points);
            }

            progress
        })
    }
}

/// How many entries (and stat points) are done out of those matching the filters. With
/// done entries hidden, shows only what remains.
#[component]
pub fn ProgressCounts(
    progress: Signal<Progress>,
    show_done: ReadSignal<bool>,
    /// Plural, e.g. "Enemies".
    items: &'static str,
    /// e.g. "killed".
    done_label: &'static str,
    /// e.g. "Bravery".
    stat: &'static str,
) -> impl IntoView {
    let counts = move || {
        let progress = progress.get();

        if show_done.get() {
            [
                (
                    format!("{items} {done_label}"),
                    format!("{}/{}", progress.done, progress.total),
                ),
                (
                    format!("{stat} earned"),
                    format!("{}/{}", progress.done_points, progress.total_points),
                ),
            ]
        } else {
            [
                (
                    format!("{items} remaining"),
                    (progress.total - progress.done).to_string(),
                ),
                (
                    format!("{stat} remaining"),
                    (progress.total_points - progress.done_points).to_string(),
                ),
            ]
        }
    };

    view! {
        <div class="counts">
            {move || {
                counts()
                    .into_iter()
                    .map(|(label, value)| {
                        view! {
                            <div class="count">
                                <span class="count-label">{label}</span>
                                <span class="count-value">{value}</span>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_adds_up_done_and_total() {
        let progress = Progress::of([(true, 2), (false, 3), (true, 1)].into_iter());

        assert_eq!(
            progress,
            Progress {
                done: 2,
                total: 3,
                done_points: 3,
                total_points: 6,
            }
        );
    }
}
