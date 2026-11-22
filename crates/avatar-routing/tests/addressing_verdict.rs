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
fn missing_addressee_answer_keeps() {
    assert_eq!(outcome(&json!([])), (Route::Keep, None, None));
}

#[test]
fn other_question_is_ignored() {
    let answers = json!([{"type": "choice", "name": "other", "choice": "Ruri", "confidence": 0.9}]);
    assert_eq!(outcome(&answers), (Route::Keep, None, None));
}

#[test]
fn non_choice_type_drops_choice_but_keeps_confidence() {
    let answers =
        json!([{"type": "text", "name": "addressee", "choice": "Ruri", "confidence": 0.9}]);
    assert_eq!(outcome(&answers), (Route::Keep, None, Some(0.9)));
}

#[test]
fn boolean_confidence_is_discarded() {
    let answers = answer(&json!("Ruri"), &json!(true));
    assert_eq!(outcome(&answers), (Route::Keep, Some("Ruri".into()), None));
}

#[test]
fn string_confidence_is_discarded() {
    let answers = answer(&json!("Ruri"), &json!("0.9"));
    assert_eq!(outcome(&answers), (Route::Keep, Some("Ruri".into()), None));
}

#[test]
fn confidence_above_one_is_discarded() {
    let answers = answer(&json!("Ruri"), &json!(1.5));
    assert_eq!(outcome(&answers), (Route::Keep, Some("Ruri".into()), None));
}

#[test]
fn negative_confidence_is_discarded() {
    let answers = answer(&json!("Ruri"), &json!(-0.1));
    assert_eq!(outcome(&answers), (Route::Keep, Some("Ruri".into()), None));
}
