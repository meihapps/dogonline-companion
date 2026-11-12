use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativityData {
    pub books: Vec<Book>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub name: String,
    pub creativity: u8,
    /// Mutable progress. Never part of the JSON data; filled in from browser storage.
    #[serde(skip)]
    pub read: bool,
}

impl CreativityData {
    /// Marks every book whose name is in `read` as read.
    pub fn apply_progress(&mut self, read: &HashSet<String>) {
        for book in &mut self.books {
            book.read = read.contains(&book.name);
        }
    }

    /// Names of all read books (what gets written to browser storage).
    pub fn read_names(&self) -> HashSet<String> {
        self.books
            .iter()
            .filter(|book| book.read)
            .map(|book| book.name.clone())
            .collect()
    }
}
