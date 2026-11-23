use std::error::Error;

use avatar_transcript::json::{JsonObject, from_value};
use avatar_transcript::transcript::report::{
    Stamp, report_addressing, report_closing, report_conversation, report_delivery, report_speech,
    report_talk, report_wake,
};
use serde_json::Value;

fn object_of(pairs: &Value) -> JsonObject {
    let mut object = JsonObject::default();
    for pair in pairs.as_array().into_iter().flatten() {
        object.set(pair[0].as_str().unwrap_or_default(), from_value(&pair[1]));
    }
    object
}

fn text(value: &Value) -> Result<&str, Box<dyn Error>> {
    Ok(value.as_str().ok_or("expected text")?)
}

fn wake_line(stamp: &Stamp, inputs: &Value) -> Result<String, Box<dyn Error>> {
    let bytes = usize::try_from(inputs["bytes"].as_u64().ok_or("bytes")?)?;
    Ok(report_wake(stamp, text(&inputs["text"])?, bytes))
}

fn pane_line(stamp: &Stamp, kind: &str, inputs: &Value) -> Result<String, Box<dyn Error>> {
    let pane = text(&inputs["pane"])?;
    Ok(match kind {
        "talk" => report_talk(stamp, text(&inputs["action"])?, pane),
        _ => report_speech(stamp, pane, &object_of(&inputs["speech"])),
    })
}

fn decision_line(stamp: &Stamp, kind: &str, inputs: &Value) -> Result<String, Box<dyn Error>> {
    Ok(match kind {
        "addressing" => {
            let decision = object_of(&inputs["decision"]);
            report_addressing(stamp, text(&inputs["phase"])?, &decision)
        }
        "delivery" => report_delivery(stamp, &object_of(&inputs["delivery"])),
        _ => report_closing(stamp, text(&inputs["reason"])?),
    })
}

pub(super) fn line_of(stamp: &Stamp, case: &Value) -> Result<String, Box<dyn Error>> {
    let inputs = &case["inputs"];
    let kind = text(&case["kind"])?;
    match kind {
        "wake" => wake_line(stamp, inputs),
        "conversation" => Ok(report_conversation(stamp, &inputs["entry"])?),
        "talk" | "speech" => pane_line(stamp, kind, inputs),
        _ => decision_line(stamp, kind, inputs),
    }
}
