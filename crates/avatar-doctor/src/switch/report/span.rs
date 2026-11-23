use super::super::moment::{Boundary, Moments};
use super::format::seconds;
use super::stage::took;

pub(super) fn recorded_span(moments: &Moments) -> String {
    let recorded: Vec<Boundary> = Boundary::ALL
        .into_iter()
        .filter(|boundary| moments.get(*boundary).is_some())
        .collect();
    match recorded.as_slice() {
        [first, .., last] => span_text(moments, *first, *last),
        _ => "nothing measurable".to_owned(),
    }
}

fn span_text(moments: &Moments, first: Boundary, last: Boundary) -> String {
    format!(
        "{} -> {} {}",
        first.label(),
        last.label(),
        seconds(took(moments, first, last))
    )
}
