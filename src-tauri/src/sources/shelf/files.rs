use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

#[derive(Default)]
pub struct ShelfFiles {
    paths: Vec<PathBuf>,
}

impl ShelfFiles {
    pub fn add_all(&mut self, dropped: Vec<PathBuf>) -> usize {
        let before = self.paths.len();
        for path in dropped {
            if !self.paths.contains(&path) {
                self.paths.push(path);
            }
        }
        self.paths.len() - before
    }

    pub fn remove(&mut self, path: &str) -> bool {
        let before = self.paths.len();
        self.paths.retain(|candidate| candidate != Path::new(path));
        self.paths.len() != before
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|candidate| candidate == Path::new(path))
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn entries(&self) -> Vec<ShelfEntry> {
        self.paths.iter().map(|path| entry_for(path)).collect()
    }
}

fn entry_for(path: &Path) -> ShelfEntry {
    let name = path.file_name().map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned());
    ShelfEntry { path: path.display().to_string(), name, is_directory: path.is_dir() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn add_all_skips_duplicates_and_counts_new_entries() {
        let mut shelf = ShelfFiles::default();
        assert_eq!(shelf.add_all(paths(&["a.txt", "b.txt"])), 2);
        assert_eq!(shelf.add_all(paths(&["b.txt", "c.txt"])), 1);
        assert_eq!(shelf.len(), 3);
    }

    #[test]
    fn remove_drops_only_the_named_path() {
        let mut shelf = ShelfFiles::default();
        shelf.add_all(paths(&["a.txt", "b.txt"]));
        assert!(shelf.remove("a.txt"));
        assert!(!shelf.remove("a.txt"));
        assert!(shelf.contains("b.txt"));
    }

    #[test]
    fn clear_empties_the_shelf() {
        let mut shelf = ShelfFiles::default();
        shelf.add_all(paths(&["a.txt"]));
        shelf.clear();
        assert!(shelf.is_empty());
    }

    #[test]
    fn entries_expose_file_names() {
        let mut shelf = ShelfFiles::default();
        shelf.add_all(paths(&["folder/report.pdf"]));
        assert_eq!(shelf.entries()[0].name, "report.pdf");
    }
}
