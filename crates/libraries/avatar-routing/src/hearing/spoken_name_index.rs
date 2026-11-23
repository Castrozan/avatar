use crate::hearing::wake_windows::push_distinct;
use crate::names::session::Session;
use crate::names::spoken_names::answered_names;
use crate::text::folding::folded;

#[must_use]
pub fn sessions_by_spoken_name(sessions: &[Session]) -> Vec<(String, Session)> {
    grouped_by_spoken_name(sessions)
        .into_iter()
        .filter_map(|(name, group)| sole_session(name, &group))
        .collect()
}

fn grouped_by_spoken_name(sessions: &[Session]) -> Vec<(String, Vec<Session>)> {
    sessions
        .iter()
        .flat_map(|session| {
            spoken_names_of(&session.servant)
                .into_iter()
                .map(move |name| (name, session))
        })
        .fold(Vec::new(), add_to_group)
}

fn sole_session(name: String, group: &[Session]) -> Option<(String, Session)> {
    match group {
        [only] => Some((name, only.clone())),
        _ => None,
    }
}

fn add_to_group(
    mut grouped: Vec<(String, Vec<Session>)>,
    (name, session): (String, &Session),
) -> Vec<(String, Vec<Session>)> {
    match grouped.iter_mut().find(|(existing, _)| *existing == name) {
        Some((_, group)) => group.push(session.clone()),
        None => grouped.push((name, vec![session.clone()])),
    }
    grouped
}

fn spoken_names_of(servant: &str) -> Vec<String> {
    answered_names(servant)
        .iter()
        .map(|name| folded(name))
        .fold(Vec::new(), push_distinct)
}
