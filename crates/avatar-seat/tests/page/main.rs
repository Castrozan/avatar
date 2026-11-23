mod checks;
mod scenario;

use std::io;

use avatar_seat::host::state::HostSnapshot;
use avatar_seat::page::{PageFiles, PageOutcome, PageRequest, PageState, SeatPage};
use checks::check;
use scenario::{Host, Request, Scenario};

struct Files(Vec<String>);

fn route_of(file: &str) -> String {
    if file == "index.html" {
        "/".to_owned()
    } else {
        format!("/{file}")
    }
}

impl PageFiles for Files {
    fn read(&self, file: &str) -> io::Result<Vec<u8>> {
        let route = route_of(file);
        let gone = self.0.contains(&route);
        gone.then(|| io::Error::other("gone"))
            .map_or(Ok(format!("body of {route}").into_bytes()), Err)
    }
}

fn ask(page: &mut SeatPage, state: &PageState<'_>, request: &Request) -> PageOutcome {
    let fetch_site = request.headers.get("Sec-Fetch-Site").cloned();
    page.answer(
        &PageRequest {
            path_and_query: request.path.clone(),
            fetch_site,
        },
        state,
    )
}

fn snapshot(host: &Host) -> HostSnapshot {
    HostSnapshot {
        listening: host.listening,
        meeting: host.meeting,
    }
}

fn replay(scenario: &Scenario) -> Vec<PageOutcome> {
    let files = Files(scenario.missing.clone());
    let state = PageState::new(scenario.attached, snapshot(&scenario.host), &files);
    let mut page = SeatPage::new(scenario.levels);
    scenario
        .requests
        .iter()
        .map(|request| ask(&mut page, &state, request))
        .collect()
}

fn reports_of(outcomes: &[PageOutcome]) -> String {
    let reports = outcomes
        .iter()
        .filter_map(|outcome| outcome.report.as_ref());
    reports.fold(String::new(), |text, report| text + report + "\n")
}

fn levels_wanted(outcomes: &[PageOutcome]) -> usize {
    outcomes
        .iter()
        .filter(|outcome| outcome.levels_wanted)
        .count()
}

fn check_scenario(scenario: &Scenario) {
    let outcomes = replay(scenario);
    assert_eq!(levels_wanted(&outcomes), scenario.levels_requested);
    assert_eq!(reports_of(&outcomes), scenario.stderr);
    let answers = outcomes.iter().map(|outcome| &outcome.answer);
    scenario
        .results
        .iter()
        .zip(answers)
        .for_each(|(expected, answer)| check(expected, answer));
}

#[test]
fn every_page_scenario_answers_as_the_python_page_did() {
    let scenarios: Vec<Scenario> =
        serde_json::from_str(include_str!("fixtures/page.json")).unwrap_or_default();
    assert!(scenarios.len() >= 15);
    scenarios.iter().for_each(check_scenario);
}
