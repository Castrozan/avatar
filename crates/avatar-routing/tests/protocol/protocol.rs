use avatar_routing::protocol::dispatch::respond;
use serde_json::{Value, json};

fn answer(line: &str) -> Value {
    respond(line, None)
}

fn answer_value(request: &Value) -> Value {
    answer(&request.to_string())
}

fn assert_error(line: &str) {
    assert!(answer(line).get("error").is_some(), "line: {line}");
}

fn verdict_request(confidence: f64) -> Value {
    json!({
        "op": "decision_verdict",
        "answers": [{"type": "choice", "name": "addressee", "choice": "Ruri", "confidence": confidence}],
        "candidates": ["Ruri", "Hokusai"]
    })
}

#[test]
fn folded_lowercases_and_strips_marks() {
    assert_eq!(
        answer(r#"{"op":"folded","text":"Ástolfo"}"#),
        json!({"ok": "astolfo"})
    );
}

#[test]
fn sound_key_reduces_word_to_its_sound() {
    assert_eq!(
        answer(r#"{"op":"sound_key","word":"chat"}"#),
        json!({"ok": "Sat"})
    );
}

#[test]
fn ratio_of_identical_texts_is_one() {
    assert_eq!(
        answer(r#"{"op":"ratio","left":"abc","right":"abc"}"#),
        json!({"ok": 1.0})
    );
}

#[test]
fn word_matches_report_text_and_end_offsets() {
    let line = r#"{"op":"word_matches","text":"hi, you"}"#;
    let expected = json!({"ok": [{"text": "hi", "end": 2}, {"text": "you", "end": 7}]});
    assert_eq!(answer(line), expected);
}

#[test]
fn confident_verdict_routes_to_candidate() {
    let expected = json!({"ok": {"route": "candidate", "choice": "Ruri", "confidence": 0.9}});
    assert_eq!(answer_value(&verdict_request(0.9)), expected);
}

#[test]
fn unsure_verdict_keeps_the_utterance() {
    let expected = json!({"ok": {"route": "keep", "choice": "Ruri", "confidence": 0.5}});
    assert_eq!(answer_value(&verdict_request(0.5)), expected);
}

#[test]
fn decision_request_carries_messages_and_addressee_question() {
    let line = r#"{"op":"decision_request","utterance":"hello","turns":[{"text":"earlier"}],"holder":null,"candidates":["Ruri"]}"#;
    let reply = answer(line);
    assert_eq!(reply["ok"]["messages"].as_array().map(Vec::len), Some(3));
    assert_eq!(reply["ok"]["questions"][0]["name"], json!("addressee"));
}

#[test]
fn request_vocabulary_overrides_the_startup_vocabulary() {
    let plain = r#"{"op":"heard_name","windows":[["Rurri"]],"names":["Ruri"]}"#;
    let ranked =
        r#"{"op":"heard_name","windows":[["Rurri"]],"names":["Ruri"],"vocabulary":{"rurri":1}}"#;
    assert_eq!(answer(plain), json!({"ok": null}));
    assert_eq!(answer(ranked), json!({"ok": {"name": "Ruri", "size": 1}}));
}

#[test]
fn vocative_session_returns_the_addressed_session() {
    let line = r#"{"op":"vocative_session","text":"Ruri, hello","sessions":[{"servant":"Ruri Hoshino","pane":"p1"}]}"#;
    assert_eq!(
        answer(line),
        json!({"ok": {"servant": "Ruri Hoshino", "pane": "p1", "host": null}})
    );
}

#[test]
fn named_in_turn_returns_the_session_and_message() {
    let line = r#"{"op":"named_in_turn","text":"hello Ruri","sessions":[{"servant":"Ruri Hoshino","pane":"p1"}]}"#;
    let session = json!({"servant": "Ruri Hoshino", "pane": "p1", "host": null});
    assert_eq!(
        answer(line),
        json!({"ok": {"session": session, "message": "hello Ruri"}})
    );
}

#[test]
fn malformed_lines_answer_with_errors() {
    [
        "not json",
        r#"{"text":"no op"}"#,
        r#"{"op":"nope"}"#,
        r#"{"op":"folded"}"#,
        r#"{"op":"ratio","left":1,"right":"x"}"#,
    ]
    .into_iter()
    .for_each(assert_error);
}
