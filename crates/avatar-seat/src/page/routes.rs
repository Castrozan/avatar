const HTML: &str = "text/html; charset=utf-8";
const JAVASCRIPT: &str = "text/javascript";
const SCRIPTS: [&str; 6] = [
    "seat.js",
    "meeting-seat.js",
    "seat-microphone.js",
    "microphone-choice.js",
    "voice-delay.js",
    "seat-audio-worklets.js",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRoute {
    pub route: String,
    pub file: &'static str,
    pub content_type: &'static str,
}

fn script_route(file: &'static str) -> PageRoute {
    PageRoute {
        route: format!("/{file}"),
        file,
        content_type: JAVASCRIPT,
    }
}

#[must_use]
pub fn seat_page_routes() -> Vec<PageRoute> {
    let index = PageRoute {
        route: "/".to_owned(),
        file: "index.html",
        content_type: HTML,
    };
    let scripts = SCRIPTS.into_iter().map(script_route);
    std::iter::once(index).chain(scripts).collect()
}
