use std::path::Path;

use avatar_mac::edge::browser::HeadlessBrowser;

use crate::fakes::{TestResult, executable, logged};

const CONVERSATION_WORDS: [&str; 12] = [
    "-g",
    "-n",
    "-a",
    "Chromium",
    "--args",
    "--headless=new",
    "--user-data-dir=/h/.local/share/avatar/conversation-browser",
    "--app=http://localhost:8772/?meeting",
    "--use-fake-ui-for-media-stream",
    "--autoplay-policy=no-user-gesture-required",
    "--no-first-run",
    "--no-default-browser-check",
];

fn browser(open: &Path) -> HeadlessBrowser {
    HeadlessBrowser {
        open: open.to_path_buf(),
        application: "Chromium".to_owned(),
        profile_flag: "--user-data-dir=/h/.local/share/avatar/conversation-browser".to_owned(),
        port: 8772,
    }
}

#[test]
fn the_headless_command_keeps_the_order_of_the_python_launcher() {
    let words = browser(Path::new("open")).arguments("?meeting", true);
    assert_eq!(words, CONVERSATION_WORDS);
}

#[test]
fn the_wake_page_has_no_autoplay_flag() {
    let words = browser(Path::new("open")).arguments("", false);
    assert!(!words.iter().any(|word| word.starts_with("--autoplay")));
}

#[test]
fn opening_runs_the_open_program_with_those_words() -> TestResult {
    let directory = tempfile::tempdir()?;
    let log = directory.path().join("log");
    let body = format!("echo \"$@\" > {}", log.display());
    let open = executable(directory.path(), "open", &body)?;
    browser(&open).open("", true)?;
    assert!(logged(&log).starts_with("-g -n -a Chromium --args --headless=new"));
    Ok(())
}
