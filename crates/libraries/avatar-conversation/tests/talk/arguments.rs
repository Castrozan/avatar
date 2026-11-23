use avatar_conversation::talk::cli::arguments::{
    SpeechArguments, TalkArguments, TalkCommand, parse_talk_arguments,
};
use avatar_conversation::talk::state::seat::seat_name;
use serde_json::{Map, Value, json};

use super::{TestResult, fixture_cases};

const FIXTURE: &str = include_str!("fixtures/arguments.json");
const THIS_MACHINE: &str = "chise";

fn agent_part(arguments: &TalkArguments) -> Value {
    let agent = arguments.open().agent;
    let resolved = agent.map(|spec| spec.resolve(Some(THIS_MACHINE)));
    json!(resolved.map(|pane| json!({"host": pane.host, "pane": pane.pane})))
}

fn open_part(arguments: &TalkArguments) -> Value {
    let open = arguments.open();
    let is_open = matches!(arguments.command, TalkCommand::Open(_));
    json!({
        "language": open.language, "character": open.character, "holder": open.holder,
        "live": is_open.then_some(open.live), "seat": open.seat.map(seat_name),
        "agent": agent_part(arguments),
    })
}

fn speech_value(speech: &SpeechArguments, wait: Option<f64>) -> Value {
    json!({"text": speech.text, "emotion": speech.emotion, "voice": speech.voice,
           "wait": wait.map(f64::to_bits)})
}

fn speech_part(command: &TalkCommand) -> Value {
    match command {
        TalkCommand::Say(speech) => speech_value(speech, None),
        TalkCommand::Ask(asking) => speech_value(&asking.speech, Some(asking.wait.wait)),
        TalkCommand::Hear(hearing) => json!({"wait": hearing.wait.to_bits()}),
        _ => json!({}),
    }
}

fn namespace(arguments: &TalkArguments) -> Value {
    let mut fields = Map::new();
    fields.insert("action".to_owned(), json!(arguments.command.name()));
    for part in [open_part(arguments), speech_part(&arguments.command)] {
        fields.extend(part.as_object().cloned().unwrap_or_default());
    }
    for key in ["text", "emotion", "voice", "wait"] {
        fields.entry(key).or_insert(Value::Null);
    }
    Value::Object(fields)
}

fn check_rejected(case: &Value, error: &clap::Error) -> TestResult {
    let expected = case["exit"].as_i64().ok_or("exit")?;
    assert_eq!(i64::from(error.exit_code()), expected, "{case}");
    Ok(())
}

fn check(case: &Value) -> TestResult {
    let argv: Vec<String> = serde_json::from_value(case["argv"].clone())?;
    match (case["accepted"].as_bool(), parse_talk_arguments(&argv)) {
        (Some(true), Ok(arguments)) => {
            assert_eq!(namespace(&arguments), case["namespace"], "{case}");
            Ok(())
        }
        (Some(false), Err(error)) => check_rejected(case, &error),
        (_, outcome) => Err(format!("{case}: {outcome:?}").into()),
    }
}

#[test]
fn accepts_and_rejects_the_vectors_argparse_does() -> TestResult {
    let cases = fixture_cases(FIXTURE)?;
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| check(case).err().map(|error| error.to_string()))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
