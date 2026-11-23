use avatar_conversation::talk::TalkFailure;

pub(crate) fn message_of(failure: Option<TalkFailure>) -> String {
    failure
        .map(|found| found.message().to_owned())
        .unwrap_or_default()
}
