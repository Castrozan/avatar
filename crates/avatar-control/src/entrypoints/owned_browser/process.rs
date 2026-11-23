use std::fs;
use std::path::Path;

use super::failure::BrowserFailure;
use super::profile::locked_process_identifier;

fn plain_digits(digits: &str) -> bool {
    let separated = digits.starts_with('_') || digits.ends_with('_') || digits.contains("__");
    !digits.is_empty()
        && !separated
        && digits
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_')
}

fn python_integer(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    let digits = trimmed.trim_start_matches(['+', '-']);
    if !plain_digits(digits) || trimmed.len() - digits.len() > 1 {
        return None;
    }
    trimmed.replace('_', "").parse().ok()
}

pub(super) fn candidate_identifiers(
    profile: &Path,
    hostname: &str,
) -> Result<Vec<Option<i64>>, BrowserFailure> {
    let mut identifiers = vec![locked_process_identifier(profile, hostname)];
    let path = profile.join("browser.pid");
    if path.exists() {
        identifiers.push(Some(read_identifier(&path)?));
    }
    Ok(identifiers)
}

fn read_identifier(path: &Path) -> Result<i64, BrowserFailure> {
    let text = fs::read_to_string(path)?;
    python_integer(&text)
        .ok_or_else(|| BrowserFailure::Value(format!("invalid literal for int(): {text:?}")))
}

pub(super) fn process_arguments(
    proc_root: &Path,
    identifier: i64,
) -> Result<Vec<Vec<u8>>, BrowserFailure> {
    let bytes = fs::read(proc_root.join(identifier.to_string()).join("cmdline"))?;
    let parts: Vec<Vec<u8>> = bytes.split(|byte| *byte == 0).map(<[u8]>::to_vec).collect();
    match parts.as_slice() {
        [first, last] if last.is_empty() => split_command(first),
        _ => Ok(parts),
    }
}

fn split_command(command: &[u8]) -> Result<Vec<Vec<u8>>, BrowserFailure> {
    let text = String::from_utf8_lossy(command);
    let words = shlex::split(&text)
        .ok_or_else(|| BrowserFailure::Value("No closing quotation".to_owned()))?;
    Ok(words.into_iter().map(String::into_bytes).collect())
}

pub(super) fn owned_process(
    profile: &Path,
    proc_root: &Path,
    hostname: &str,
) -> Result<Option<i64>, BrowserFailure> {
    let marker = format!("--user-data-dir={}", profile.display()).into_bytes();
    let identifiers = candidate_identifiers(profile, hostname)?;
    let owner = identifiers.into_iter().flatten().find_map(|identifier| {
        owns(proc_root, identifier, &marker)
            .map(|owned| owned.then_some(identifier))
            .transpose()
    });
    owner.transpose()
}

fn owns(proc_root: &Path, identifier: i64, marker: &[u8]) -> Result<bool, BrowserFailure> {
    match process_arguments(proc_root, identifier) {
        Ok(arguments) => Ok(arguments.iter().any(|argument| argument == marker)),
        Err(failure) if failure.is_missing_file() => Ok(false),
        Err(failure) => Err(failure),
    }
}
