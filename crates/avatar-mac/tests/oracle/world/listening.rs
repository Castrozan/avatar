use avatar_listening::conversation_state::{ConversationPhase, ListenerPurpose};
use avatar_mac::session::ListeningPlan;
use serde_json::{Map, Value, json};

pub(super) fn phase_of(name: &str) -> ConversationPhase {
    match name {
        "meeting" => ConversationPhase::Meeting,
        "open" => ConversationPhase::Open,
        "live" => ConversationPhase::Live,
        "interrupted" => ConversationPhase::Interrupted,
        _ => ConversationPhase::Closed,
    }
}

pub(super) fn purpose_name(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Meeting => "meeting",
        ListenerPurpose::Conversation => "conversation",
        ListenerPurpose::Live => "live",
    }
}

pub(super) fn sources_of(plan: &ListeningPlan) -> Map<String, Value> {
    plan.sources
        .iter()
        .map(|(name, source)| (name.clone(), json!(source)))
        .collect()
}

pub(super) fn flag_entry(flag: &(&str, bool)) -> Option<(String, Value)> {
    flag.1.then(|| (flag.0.to_owned(), json!(true)))
}

pub(super) fn settings_of(plan: &ListeningPlan) -> Map<String, Value> {
    let flags = [
        ("identify_speakers", plan.identify_speakers),
        ("barge_in", plan.barge_in),
    ];
    let mut settings = Map::from_iter([("language".to_owned(), json!(plan.language))]);
    settings.extend(flags.iter().filter_map(flag_entry));
    settings
}
