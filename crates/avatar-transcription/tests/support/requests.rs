use super::case::{Expected, decode_hex};
use super::mock::RecordedRequest;
use super::normalize::{boundary_of, replace_boundary_text, replace_bytes};

const IGNORED_HEADERS: [&str; 6] = [
    "host",
    "accept-encoding",
    "user-agent",
    "connection",
    "accept",
    "content-length",
];

pub(crate) fn request_mismatches(
    name: &str,
    expected: &[Expected],
    actual: &[RecordedRequest],
) -> Vec<String> {
    let mut found = request_count_mismatch(name, expected.len(), actual.len());
    let pairs = expected.iter().zip(actual).enumerate();
    found.extend(
        pairs.flat_map(|(index, (want, got))| one_request_mismatches(name, index, want, got)),
    );
    found
}

fn request_count_mismatch(name: &str, expected: usize, actual: usize) -> Vec<String> {
    if expected == actual {
        return Vec::new();
    }
    vec![format!("{name}: {actual} requests, expected {expected}")]
}

fn one_request_mismatches(
    name: &str,
    index: usize,
    want: &Expected,
    got: &RecordedRequest,
) -> Vec<String> {
    let label = format!("{name} request #{index}");
    let mut problems = Vec::new();
    problems.extend(route_problem(want, got));
    problems.extend(header_problem(want, got));
    problems.extend(body_problem(want, got));
    problems
        .into_iter()
        .map(|problem| format!("{label}: {problem}"))
        .collect()
}

fn route_problem(want: &Expected, got: &RecordedRequest) -> Option<String> {
    (want.method != got.method || want.path != got.target)
        .then(|| format!("route {} {}", got.method, got.target))
}

fn header_problem(want: &Expected, got: &RecordedRequest) -> Option<String> {
    let (headers, _) = normalized_request(got);
    (sorted(comparable_headers(&want.headers)) != sorted(headers))
        .then(|| format!("headers {:?}", got.headers))
}

fn body_problem(want: &Expected, got: &RecordedRequest) -> Option<String> {
    let (_, body) = normalized_request(got);
    (decode_hex(&want.body_hex) != body).then(|| "body differs".to_owned())
}

fn comparable_headers(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| !IGNORED_HEADERS.contains(&name.as_str()))
        .cloned()
        .collect()
}

fn sorted(mut headers: Vec<(String, String)>) -> Vec<(String, String)> {
    headers.sort();
    headers
}

fn normalized_request(request: &RecordedRequest) -> (Vec<(String, String)>, Vec<u8>) {
    let boundary = boundary_of(request);
    let headers = comparable_headers(&request.headers)
        .into_iter()
        .map(|(name, value)| (name, replace_boundary_text(&value, &boundary)))
        .collect();
    (headers, replace_bytes(&request.body, &boundary))
}
