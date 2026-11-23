use serde_json::json;

use super::FAILURE_PATH;
use super::HEALTH_PATH;
use super::LEVELS_PATH;
use super::answer::{PageAnswer, PageOutcome};
use super::decoding::percent_decoded;
use super::request::{PageRequest, PageState};
use super::routes::{PageRoute, seat_page_routes};

const SAME_ORIGIN: &str = "same-origin";

pub struct SeatPage {
    routes: Vec<PageRoute>,
    levels_enabled: bool,
    pub failure: Option<String>,
}

fn empty_text() -> PageOutcome {
    PageOutcome::of(PageAnswer::body(Vec::new(), "text/plain"))
}

fn forbidden() -> PageOutcome {
    PageOutcome::of(PageAnswer::Status(403))
}

fn request_levels(fetch_site: Option<&str>) -> PageOutcome {
    if fetch_site.unwrap_or(SAME_ORIGIN) != SAME_ORIGIN {
        return forbidden();
    }
    PageOutcome {
        levels_wanted: true,
        ..empty_text()
    }
}

fn serve_file(routes: &[PageRoute], path: &str, state: &PageState<'_>) -> PageOutcome {
    let Some(route) = routes.iter().find(|candidate| candidate.route == path) else {
        return PageOutcome::of(PageAnswer::Status(404));
    };
    match state.files.read(route.file) {
        Ok(body) => PageOutcome::of(PageAnswer::body(body, route.content_type)),
        Err(_) => PageOutcome::of(PageAnswer::Status(500)),
    }
}

fn report_health(failure: Option<&String>, state: &PageState<'_>) -> PageOutcome {
    let (listening, meeting) = (state.host.listening, state.host.meeting);
    let health = json!({"attached": state.attached, "failure": failure, "listening": listening, "meeting": meeting});
    PageOutcome::of(PageAnswer::body(
        health.to_string().into_bytes(),
        "application/json",
    ))
}

fn record_failure(page: &mut SeatPage, query: &str, fetch_site: Option<&str>) -> PageOutcome {
    if fetch_site != Some(SAME_ORIGIN) {
        return forbidden();
    }
    let failure = percent_decoded(query);
    let report = format!("The seat page failed: {failure}");
    page.failure = Some(failure);
    PageOutcome {
        report: Some(report),
        ..empty_text()
    }
}

fn split_query(path_and_query: &str) -> (&str, &str) {
    path_and_query
        .split_once('?')
        .unwrap_or((path_and_query, ""))
}

impl SeatPage {
    #[must_use]
    pub fn new(levels_enabled: bool) -> Self {
        Self {
            routes: seat_page_routes(),
            levels_enabled,
            failure: None,
        }
    }

    pub fn answer(&mut self, request: &PageRequest, state: &PageState<'_>) -> PageOutcome {
        let (path, query) = split_query(&request.path_and_query);
        let fetch_site = request.fetch_site.as_deref();
        match path {
            HEALTH_PATH => report_health(self.failure.as_ref(), state),
            FAILURE_PATH => record_failure(self, query, fetch_site),
            LEVELS_PATH if self.levels_enabled => request_levels(fetch_site),
            _ => serve_file(&self.routes, path, state),
        }
    }
}
