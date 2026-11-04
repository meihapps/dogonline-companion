pub fn format_enum<T: std::fmt::Debug>(item: &T) -> String {
    const LOWERCASE_WORDS: &[&str] = &["of"];

    let name = format!("{item:?}");
    let mut words = Vec::new();
    let mut current_word = String::new();

    for ch in name.chars() {
        if ch.is_uppercase() && !current_word.is_empty() {
            words.push(std::mem::take(&mut current_word));
        }

        current_word.push(ch.to_ascii_lowercase());
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            if index == 0 || !LOWERCASE_WORDS.contains(&word.as_str()) {
                let mut chars = word.chars();

                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            } else {
                word.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_list<T: std::fmt::Debug>(items: &[T]) -> String {
    items.iter().map(format_enum).collect::<Vec<_>>().join(", ")
}

/// Where the image for `name` is served from. The file names are slugs of the item names,
/// written by `scripts/fetch_wiki_assets.py`.
pub fn image_path(tab: &str, name: &str) -> String {
    let mut slug = String::new();

    for ch in name.chars().filter(|ch| *ch != '\'') {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }

    format!("wiki/{tab}/{}.png", slug.trim_end_matches('-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordination::{Bait, Weather};

    #[test]
    fn format_enum_splits_words_and_keeps_of_lowercase() {
        assert_eq!(format_enum(&Bait::BucketOfWorms), "Bucket of Worms");
        assert_eq!(format_enum(&Weather::ClearSkies), "Clear Skies");
    }

    #[test]
    fn format_list_joins_with_commas() {
        assert_eq!(
            format_list(&[Weather::Rainy, Weather::ClearSkies]),
            "Rainy, Clear Skies"
        );
    }

    #[test]
    fn image_path_slugifies_names() {
        assert_eq!(
            image_path("bravery", "Cheese (Enemy)"),
            "wiki/bravery/cheese-enemy.png"
        );
        assert_eq!(
            image_path("bravery", "Growed-Up Pieder"),
            "wiki/bravery/growed-up-pieder.png"
        );
    }
}
