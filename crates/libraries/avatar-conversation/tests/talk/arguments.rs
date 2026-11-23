use avatar_conversation::talk::cli::arguments::{TalkArguments, TalkCommand, parse_talk_arguments};
use serde_json::{Value, json};

use super::{TestResult, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/arguments.json");
const THIS_MACHINE: &str = "chise";

fn namespace(arguments: &TalkArguments) -> Value {
    let open = arguments.open();
    let (text, emotion, voice, wait) = speech_fields(&arguments.command);
    let agent = open
        .agent
        .as_ref()
        .map(|spec| spec.resolve(Some(THIS_MACHINE)));
    json!({
        "action": arguments.command.name(),
        "language": open.language, "character": open.character,
        "live": matches!(arguments.command, TalkCommand::Open(_)).then_some(open.live),
        "holder": open.holder,
        "seat": open.seat.map(avatar_conversation::talk::state::seat::seat_name),
        "agent": agent.map(|pane| json!({"host": pane.host, "pane": pane.pane})),
        "text": text, "emotion": emotion, "voice": voice, "wait": wait.map(f64::to_bits),
    })
}

fn speech_fields(command: &TalkCommand) -> (Option<&str>, Option<&str>, Option<&str>, Option<f64>) {
    match command {
        TalkCommand::Say(speech) => fields_of(speech, None),
        TalkCommand::Ask(asking) => fields_of(&asking.speech, Some(asking.wait.wait)),
        TalkCommand::Hear(hearing) => (None, None, None, Some(hearing.wait)),
        _ => (None, None, None, None),
    }
}

fn fields_of(
    speech: &avatar_conversation::talk::cli::arguments::SpeechArguments,
    wait: Option<f64>,
) -> (Option<&str>, Option<&str>, Option<&str>, Option<f64>) {
    (
        Some(&speech.text),
        Some(&speech.emotion),
        speech.voice.as_deref(),
        wait,
    )
}

fn check(case: &Value) -> TestResult {
    let argv: Vec<String> = serde_json::from_value(case["argv"].clone())?;
    let parsed = parse_talk_arguments(&argv);
    match (case["accepted"].as_bool(), parsed) {
        (Some(true), Ok(arguments)) => {
            assert_eq!(namespace(&arguments), case["namespace"], "{case}")
        }
        (Some(false), Err(error)) => assert_eq!(
            i64::from(error.exit_code()),
            case["exit"].as_i64().ok_or("exit")?,
            "{case}"
        ),
        (_, outcome) => return Err(format!("{case}: {outcome:?}").into()),
    }
    Ok(())
}

#[test]
fn accepts_and_rejects_the_vectors_argparse_does() -> TestResult {
    let failures: Vec<String> = fixture_cases(FIXTURE)?
        .iter()
        .filter_map(|case| {
            check(case)
                .err()
                .map(|error| format!("{}: {error}", case["argv"]))
        })
        .map(|text| text.chars().take(300).collect())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
