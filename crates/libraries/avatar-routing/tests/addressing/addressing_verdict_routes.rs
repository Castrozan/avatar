use avatar_routing::addressing::decision::{Route, verdict};
use serde_json::{Value, json};

fn candidates() -> Vec<String> {
    vec!["Ruri".to_owned(), "Hokusai".to_owned()]
}

fn answer(choice: &Value, confidence: &Value) -> Value {
    json!([{"type": "choice", "name": "addressee", "choice": choice, "confidence": confidence}])
}

fn outcome(answers: &Value) -> (Route, Option<String>, Option<f64>) {
    let decided = verdict(answers, &candidates());
    (decided.route, decided.choice, decided.confidence)
}

#[test]
fn confidence_below_floor_keeps_with_value_reported() {
    let answers = answer(&json!("Ruri"), &json!(0.79));
    assert_eq!(
        outcome(&answers),
        (Route::Keep, Some("Ruri".into()), Some(0.79))
    );
}

#[test]
fn confidence_at_floor_routes_to_candidate() {
    let answers = answer(&json!("Ruri"), &json!(0.8));
    assert_eq!(
        outcome(&answers),
        (Route::Candidate, Some("Ruri".into()), Some(0.8))
    );
}

#[test]
fn full_confidence_routes_to_candidate() {
    let answers = answer(&json!("Hokusai"), &json!(1));
    assert_eq!(
        outcome(&answers),
        (Route::Candidate, Some("Hokusai".into()), Some(1.0))
    );
}

#[test]
fn nobody_choice_routes_to_nobody() {
    let answers = answer(&json!("nobody"), &json!(0.1));
    assert_eq!(
        outcome(&answers),
        (Route::Nobody, Some("nobody".into()), Some(0.1))
    );
}

#[test]
fn non_candidate_choice_keeps_even_when_confident() {
    let answers = answer(&json!("Zed"), &json!(0.95));
    assert_eq!(
        outcome(&answers),
        (Route::Keep, Some("Zed".into()), Some(0.95))
    );
}

#[test]
fn holder_choice_keeps() {
    let answers = answer(&json!("holder"), &json!(0.95));
    assert_eq!(
        outcome(&answers),
        (Route::Keep, Some("holder".into()), Some(0.95))
    );
}

#[test]
fn first_addressee_entry_wins() {
    let answers = json!([
        {"type": "choice", "name": "addressee", "choice": "Hokusai", "confidence": 0.9},
        {"type": "choice", "name": "addressee", "choice": "Ruri", "confidence": 0.95}
    ]);
    assert_eq!(
        outcome(&answers),
        (Route::Candidate, Some("Hokusai".into()), Some(0.9))
    );
}
