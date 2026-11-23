#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestTarget {
    pub raw: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

fn collapse_leading_slashes(raw: &str) -> String {
    if raw.starts_with("//") {
        return format!("/{}", raw.trim_start_matches('/'));
    }
    raw.to_owned()
}

fn prefixed(prefix: char, text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    format!("{prefix}{text}")
}

fn split_target(raw: &str) -> (String, String, String) {
    let (before_fragment, fragment) = raw.split_once('#').unwrap_or((raw, ""));
    let (path, query) = before_fragment
        .split_once('?')
        .unwrap_or((before_fragment, ""));
    (path.to_owned(), query.to_owned(), fragment.to_owned())
}

impl RequestTarget {
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let raw = collapse_leading_slashes(raw);
        let (path, query, fragment) = split_target(&raw);
        Self {
            raw,
            path,
            query,
            fragment,
        }
    }

    #[must_use]
    pub fn has_trailing_slash(&self) -> bool {
        self.path.trim_end().ends_with('/')
    }

    #[must_use]
    pub fn directory_location(&self) -> String {
        let query = prefixed('?', &self.query);
        format!("{}/{query}{}", self.path, prefixed('#', &self.fragment))
    }
}
