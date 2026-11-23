pub(super) fn named(name: Option<&str>) -> Option<&str> {
    name.filter(|text| !text.is_empty())
}

pub(super) fn is_another_speaker(incoming: Option<&str>, current: Option<&str>) -> bool {
    match (named(incoming), named(current)) {
        (Some(incoming_name), Some(current_name)) => incoming_name != current_name,
        _ => false,
    }
}
