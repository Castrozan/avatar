use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use avatar_seat::page::{PageRoute, seat_page_routes};

const HTML: &str = "text/html; charset=utf-8";
const JAVASCRIPT: &str = "text/javascript";
const WAKE_PAGE_FILES: [(&str, &str, &str); 2] = [
    ("/", "index.html", HTML),
    ("/wake-capture.js", "wake-capture.js", JAVASCRIPT),
];
const SHARED_SCRIPTS: [&str; 3] = [
    "seat-microphone.js",
    "microphone-choice.js",
    "seat-audio-worklets.js",
];

#[derive(Debug, Default)]
pub struct PageFiles {
    entries: BTreeMap<String, (PathBuf, &'static str)>,
}

type Entry = (String, (PathBuf, &'static str));

fn seat_entry(directory: &Path, route: PageRoute) -> Entry {
    let file = directory.join(route.file);
    (route.route, (file, route.content_type))
}

fn named_entry(directory: &Path, route: &str, file: &str, kind: &'static str) -> Entry {
    (route.to_owned(), (directory.join(file), kind))
}

fn wake_entries(directory: &Path) -> impl Iterator<Item = Entry> {
    let entry = |(route, file, kind)| named_entry(directory, route, file, kind);
    WAKE_PAGE_FILES.into_iter().map(entry)
}

fn script_entries(directory: &Path) -> impl Iterator<Item = Entry> {
    let entry = |file: &str| named_entry(directory, &format!("/{file}"), file, JAVASCRIPT);
    SHARED_SCRIPTS.into_iter().map(entry)
}

impl PageFiles {
    #[must_use]
    pub fn seat(directory: &Path) -> Self {
        let routes = seat_page_routes().into_iter();
        let entries = routes.map(|route| seat_entry(directory, route)).collect();
        Self { entries }
    }

    #[must_use]
    pub fn microphone(wake_directory: &Path, seat_directory: &Path) -> Self {
        let wake = wake_entries(wake_directory);
        let entries = wake.chain(script_entries(seat_directory)).collect();
        Self { entries }
    }

    #[must_use]
    pub fn lookup(&self, path: &str) -> Option<&(PathBuf, &'static str)> {
        self.entries.get(path)
    }
}
