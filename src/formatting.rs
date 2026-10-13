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
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
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
    items
        .iter()
        .map(format_enum)
        .collect::<Vec<_>>()
        .join(", ")
}
