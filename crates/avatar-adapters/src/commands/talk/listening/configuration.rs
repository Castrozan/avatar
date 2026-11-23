use avatar_conversation::talk::ports::ListeningRequest;
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, Value, json};

pub(super) const LISTENER_SERVICE: &str = "avatar-listener.service";
pub(super) const LIVE_SERVICE: &str = "avatar-live.service";

fn purpose_name(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Conversation => "conversation",
        ListenerPurpose::Meeting => "meeting",
        ListenerPurpose::Live => "live",
    }
}

pub(super) fn service_of(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Live => LIVE_SERVICE,
        _ => LISTENER_SERVICE,
    }
}

pub(super) fn configuration_text(request: &ListeningRequest) -> String {
    let mut document = Map::new();
    document.insert("purpose".to_owned(), json!(purpose_name(request.purpose)));
    document.insert("sources".to_owned(), json!({"human": request.human_source}));
    document.extend(request.settings.clone());
    Value::Object(document).to_string()
}
