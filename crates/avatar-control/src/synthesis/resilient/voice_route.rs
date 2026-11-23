pub(super) enum VoiceRoute {
    Piper,
    Kokoro,
    Cloud,
}

pub(super) fn route_of(voice: Option<&str>) -> VoiceRoute {
    match voice {
        Some(name) if is_piper_voice(name) => VoiceRoute::Piper,
        Some(name) if is_kokoro_voice(name) => VoiceRoute::Kokoro,
        _ => VoiceRoute::Cloud,
    }
}

fn is_piper_voice(name: &str) -> bool {
    matches!(name.as_bytes(), [first, second, b'_', third, fourth, b'-', ..]
        if first.is_ascii_lowercase() && second.is_ascii_lowercase() && third.is_ascii_uppercase() && fourth.is_ascii_uppercase())
}

fn is_kokoro_voice(name: &str) -> bool {
    matches!(name.as_bytes(), [first, second, b'_', rest @ ..]
        if first.is_ascii_lowercase() && second.is_ascii_lowercase() && !rest.is_empty() && rest.iter().all(u8::is_ascii_lowercase))
}
