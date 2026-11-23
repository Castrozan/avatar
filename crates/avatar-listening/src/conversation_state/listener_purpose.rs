use super::ListenerPurpose;

const CAPTURE_UNITS: [&str; 2] = ["avatar-listener.service", "avatar-live.service"];

pub trait ListenerProbe {
    fn unit_active(&mut self, unit: &str) -> bool;
    fn configured_purpose(&mut self) -> Option<String>;
}

#[must_use]
pub fn live_purpose(probe: &mut impl ListenerProbe) -> Option<ListenerPurpose> {
    if !listening(probe) {
        return None;
    }
    purpose_named(probe.configured_purpose().as_deref())
}

fn listening(probe: &mut impl ListenerProbe) -> bool {
    CAPTURE_UNITS.iter().any(|unit| probe.unit_active(unit))
}

fn purpose_named(name: Option<&str>) -> Option<ListenerPurpose> {
    match name? {
        "conversation" => Some(ListenerPurpose::Conversation),
        "meeting" => Some(ListenerPurpose::Meeting),
        "live" => Some(ListenerPurpose::Live),
        _ => None,
    }
}
