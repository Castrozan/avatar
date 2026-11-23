use super::record::Record;

pub(crate) const CONVERSATION: &str = "conversation";

#[must_use]
pub(crate) fn is_delivery(record: &Record) -> bool {
    record.is_text("phase", "delivery")
}

#[must_use]
pub(crate) fn is_switch_decision(record: &Record) -> bool {
    record.is_text("decision", "addressing")
        && record.is_text("phase", CONVERSATION)
        && record.is_text("verdict", "candidate")
}

#[must_use]
pub(crate) fn is_conversation_switch(record: &Record) -> bool {
    is_switch_decision(record) || record.is_text("purpose", CONVERSATION)
}

#[must_use]
pub(crate) fn is_anchor(record: &Record) -> bool {
    is_delivery(record) || is_switch_decision(record)
}

pub(crate) fn heard_as(sequence: Option<i64>) -> impl Fn(&Record) -> bool {
    move |record: &Record| {
        record.is_text("phase", CONVERSATION) && record.has("text") && record.sequence() == sequence
    }
}

pub(crate) fn decided_on(sequence: Option<i64>) -> impl Fn(&Record) -> bool {
    move |record: &Record| record.is_text("decision", "addressing") && record.sequence() == sequence
}

pub(crate) fn talk_from(pane: Option<&str>) -> impl Fn(&Record) -> bool + '_ {
    move |record: &Record| record.is_text("phase", "talk") && record.text("pane") == pane
}

pub(crate) fn reply_from(
    pane: Option<&str>,
    delivered: f64,
    tenure_end: f64,
) -> impl Fn(&Record) -> bool + '_ {
    move |record: &Record| {
        record.is_text("phase", "speech")
            && record.text("pane") == pane
            && record.is_recorded("playing")
            && invoked_within(record, delivered, tenure_end)
    }
}

fn invoked_within(record: &Record, delivered: f64, tenure_end: f64) -> bool {
    let invoked = record.number("invoked").unwrap_or(f64::NEG_INFINITY);
    delivered <= invoked && invoked < tenure_end
}
