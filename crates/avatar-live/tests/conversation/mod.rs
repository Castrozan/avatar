mod fakes;
mod hand_off;
mod outcomes;
mod scripted;

use avatar_live::conversation::Conversation;
use serde_json::{Value, json};

use self::fakes::{Recorder, Screen, Speaker, TurnRecord};
use crate::support::{Outcome, array, event_from, number, plain, same, text, try_cases};

const FIXTURE: &str = include_str!("fixtures/conversation.json");

type Subject = Conversation<Speaker, TurnRecord, Screen>;

fn subject_for(recorder: &Recorder) -> Subject {
    let parts = (
        Speaker(recorder.clone()),
        TurnRecord(recorder.clone()),
        Screen(recorder.clone()),
    );
    Conversation::new(parts.0, parts.1, parts.2)
}

fn hear(subject: &mut Subject, recorder: &Recorder, operation: &Value) -> Outcome {
    let task = subject.handle(event_from(&operation[2])?, number(&operation[1]));
    if let Some(found) = task {
        recorder.push(json!(["handoff", found.task_id, found.request]));
    }
    Ok(())
}

fn apply(subject: &mut Subject, recorder: &Recorder, operation: &Value) -> Outcome {
    match text(&operation[0]) {
        "event" => hear(subject, recorder, operation)?,
        "settle" => subject.settle(number(&operation[1])),
        "buffer" => recorder.buffered.set(number(&operation[1])),
        "voice_changed" => subject.settle_after_a_new_voice(),
        _ => subject.finish(),
    }
    recorder.push(json!(["--"]));
    Ok(())
}

fn summary_matches(subject: &Subject, case: &Value) -> Outcome {
    same(
        &plain(&subject.summary("fake-live"))?,
        &case["summary"],
        case,
    );
    Ok(())
}

fn matches_python(subject: &Subject, recorder: &Recorder, case: &Value) -> Outcome {
    same(&json!(recorder.log.borrow().clone()), &case["log"], case);
    same(
        &json!(recorder.printed.borrow().clone()),
        &case["printed"],
        case,
    );
    summary_matches(subject, case)
}

fn check(case: &Value) -> Outcome {
    let recorder = Recorder::default();
    let mut subject = subject_for(&recorder);
    array(&case["ops"])
        .iter()
        .try_for_each(|operation| apply(&mut subject, &recorder, operation))?;
    matches_python(&subject, &recorder, case)
}

#[test]
fn every_recorded_python_conversation_is_reproduced() -> Outcome {
    try_cases(FIXTURE, 17, check)
}
