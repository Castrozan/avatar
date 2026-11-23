use std::fs;
use std::path::Path;

use super::page::escape_html;
use super::translate::percent_decode;

const SAFE_BYTES: &str = "_.-~/";

fn quoted_byte(byte: u8) -> String {
    let safe = byte.is_ascii_alphanumeric() || SAFE_BYTES.contains(char::from(byte));
    if safe {
        return char::from(byte).to_string();
    }
    format!("%{byte:02X}")
}

#[must_use]
pub fn quote(text: &str) -> String {
    text.bytes().map(quoted_byte).collect()
}

struct Entry {
    name: String,
    display: String,
    link: String,
}

fn marker(condition: bool, mark: &str) -> String {
    if condition {
        return mark.to_owned();
    }
    String::new()
}

fn entry_for(directory: &Path, name: String) -> Entry {
    let full = directory.join(&name);
    let slash = marker(full.is_dir(), "/");
    let mark = marker(full.is_symlink(), "@");
    let display_mark = if mark.is_empty() { &slash } else { &mark };
    Entry {
        display: format!("{name}{display_mark}"),
        link: format!("{name}{slash}"),
        name,
    }
}

fn entries(directory: &Path) -> Vec<Entry> {
    let names = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|item| item.file_name().to_string_lossy().into_owned());
    let mut found: Vec<Entry> = names.map(|name| entry_for(directory, name)).collect();
    found.sort_by_key(|entry| entry.name.to_lowercase());
    found
}

fn item_html(entry: &Entry) -> String {
    let link = quote(&entry.link);
    format!(
        "<li><a href=\"{link}\">{}</a></li>\n",
        escape_html(&entry.display)
    )
}

pub(super) fn listing_html(directory: &Path, raw_target: &str) -> String {
    let title = format!(
        "Directory listing for {}",
        escape_html(&percent_decode(raw_target))
    );
    let items: String = entries(directory).iter().map(item_html).collect();
    let head = format!(
        "<!DOCTYPE HTML>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n</head>\n"
    );
    format!("{head}<body>\n<h1>{title}</h1>\n<hr>\n<ul>\n{items}</ul>\n<hr>\n</body>\n</html>\n")
}
