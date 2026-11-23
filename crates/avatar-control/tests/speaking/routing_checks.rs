use avatar_control::speaking::{SpeakRequest, output_plan};
use serde_json::{Value, json};

use crate::trace::{expected_sinks, field, observed_sinks};

pub(crate) fn check_generation(name: &str, request: &SpeakRequest, generation: &Value) {
    check_identity(name, request, generation);
    check_options(name, request, generation);
}

fn check_identity(name: &str, request: &SpeakRequest, generation: &Value) {
    assert_eq!(
        field(generation, "outputId"),
        request.output_id(0).as_str(),
        "{name}"
    );
    assert_eq!(field(generation, "text"), request.text.as_str(), "{name}");
}

fn check_options(name: &str, request: &SpeakRequest, generation: &Value) {
    assert_eq!(field(generation, "voice"), &json!(request.voice), "{name}");
    assert_eq!(
        field(generation, "emotion"),
        &json!(request.emotion),
        "{name}"
    );
    assert_eq!(field(generation, "model"), &json!(request.model), "{name}");
}

pub(crate) fn check_routing(name: &str, request: &SpeakRequest, trace: &[Value]) {
    let plan = output_plan(&request.output);
    assert_eq!(observed_sinks(trace), expected_sinks(&plan.route), "{name}");
    let opened = trace
        .iter()
        .any(|entry| entry.get("channel") == Some(&json!("open")));
    assert_eq!(opened, plan.uses_agent_channel, "{name}");
}
