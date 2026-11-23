mod fakes;
mod files;

use std::path::Path;

use crate::support::{Outcome, fixture, rows, text};
use avatar_listening::listener::cli::{ListenArguments, ListenFailure};
use avatar_listening::listener::session::SessionDirectory;
use avatar_listening::listener::speaker_identity::{VoiceProfiles, append_embedding};
use clap::Parser;
use files::assert_voice_files;
use serde_json::Value;

fn floats_of(hexes: &Value) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let parse = |hex: &Value| Ok(f64::from_bits(u64::from_str_radix(text(hex)?, 16)?));
    rows(hexes)?.iter().map(parse).collect()
}

fn seed(directory: &Path, scenario: &Value) -> Outcome {
    for entry in rows(&scenario["entries"])? {
        let sequence = entry[0].as_i64().ok_or("sequence")?;
        append_embedding(
            &directory.join("embeddings.jsonl"),
            sequence,
            &floats_of(&entry[1])?,
        )?;
    }
    Ok(())
}

struct Ran {
    code: i64,
    lines: Vec<String>,
    stderr: String,
}

fn parsed(argv: &Value) -> Result<Option<ListenArguments>, Box<dyn std::error::Error>> {
    let mut words = vec!["avatar-listen".to_owned()];
    words.extend(
        rows(argv)?
            .iter()
            .map(|word| text(word).unwrap_or_default().to_owned()),
    );
    Ok(ListenArguments::try_parse_from(words).ok())
}

fn message_of(failure: Option<&ListenFailure>) -> String {
    failure.map(ToString::to_string).unwrap_or_default()
}

fn execute(arguments: &ListenArguments, directory: &Path, voices: &VoiceProfiles) -> Ran {
    let session = SessionDirectory::new(directory);
    let (failure, lines) = fakes::run_command(&arguments.command, &session, voices);
    let code = i64::from(failure.is_some());
    Ran {
        code,
        lines,
        stderr: message_of(failure.as_ref()),
    }
}

fn step(
    directory: &Path,
    voices: &VoiceProfiles,
    argv: &Value,
) -> Result<Ran, Box<dyn std::error::Error>> {
    let Some(arguments) = parsed(argv)? else {
        return Ok(Ran {
            code: 2,
            lines: Vec::new(),
            stderr: String::new(),
        });
    };
    Ok(execute(&arguments, directory, voices))
}

fn assert_step(ran: &Ran, expected: &Value) -> Outcome {
    let stdout: Result<Vec<String>, _> = rows(&expected["stdout"])?
        .iter()
        .map(|line| text(line).map(str::to_owned))
        .collect();
    assert_eq!(Some(ran.code), expected["code"].as_i64(), "{expected}");
    assert_eq!(ran.lines, stdout?, "{expected}");
    assert_eq!(ran.stderr, text(&expected["stderr"])?, "{expected}");
    Ok(())
}

fn replay(scenario: &Value) -> Outcome {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("run");
    std::fs::create_dir(&directory)?;
    seed(&directory, scenario)?;
    let voices = VoiceProfiles::new(root.path().join("state/avatar/voices"));
    for expected in rows(&scenario["steps"])? {
        assert_step(&step(&directory, &voices, &expected["argv"])?, expected)?;
    }
    assert_voice_files(root.path(), scenario)
}

#[test]
fn enroll_voices_and_forget_match_the_python_cli() -> Outcome {
    let document = fixture("edges/cli/voices/fixtures/voices_cli.json")?;
    let scenarios = rows(&document["scenarios"])?;
    assert_eq!(scenarios.len(), 4);
    scenarios.iter().try_for_each(replay)
}
