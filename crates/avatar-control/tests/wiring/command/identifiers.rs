use avatar_control::command::{format_version_four, random_identifier};
use serde_json::Value;

use crate::support::{TestResult, check};

pub(crate) fn check_identifier(case: &Value) -> TestResult {
    let hex = case["bytes"].as_str().unwrap_or_default();
    let mut bytes = [0_u8; 16];
    for (slot, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair)?, 16)?;
    }
    check(
        format_version_four(bytes) == case["uuid"].as_str().unwrap_or_default(),
        case,
    )
}

#[test]
pub(crate) fn identifiers_are_formatted_like_python_uuid_version_four() -> TestResult {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/uuid_cases.json"))?;
    cases.iter().try_for_each(check_identifier)
}

#[test]
pub(crate) fn random_identifiers_differ_and_have_the_uuid_shape() -> TestResult {
    let first = random_identifier()?;
    let second = random_identifier()?;
    check(
        first != second && first.len() == 36 && first.as_bytes()[14] == b'4',
        &first,
    )
}
