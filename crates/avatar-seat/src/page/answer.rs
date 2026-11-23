#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageAnswer {
    Body { content_type: String, body: Vec<u8> },
    Status(u16),
}

fn header(name: &str, value: String) -> (String, String) {
    (name.to_owned(), value)
}

impl PageAnswer {
    #[must_use]
    pub fn body(body: Vec<u8>, content_type: &str) -> Self {
        Self::Body {
            content_type: content_type.to_owned(),
            body,
        }
    }

    #[must_use]
    pub fn headers(&self) -> Vec<(String, String)> {
        match self {
            Self::Body { content_type, body } => body_headers(content_type, body.len()),
            Self::Status(_) => Vec::new(),
        }
    }
}

fn body_headers(content_type: &str, length: usize) -> Vec<(String, String)> {
    vec![
        header("Content-Type", content_type.to_owned()),
        header("Content-Length", length.to_string()),
        header("Cache-Control", "no-store".to_owned()),
        header("Connection", "close".to_owned()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageOutcome {
    pub answer: PageAnswer,
    pub levels_wanted: bool,
    pub report: Option<String>,
}

impl PageOutcome {
    #[must_use]
    pub fn of(answer: PageAnswer) -> Self {
        Self {
            answer,
            levels_wanted: false,
            report: None,
        }
    }
}
