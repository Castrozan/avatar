use super::super::boundaries::WakeEffects;
use super::super::failure::Boundary;

const TRANSCRIBER_ATTEMPTS: usize = 5;
const TRANSCRIBER_RETRY_SECONDS: f64 = 0.4;

fn attempt(effects: &dyn WakeEffects, segment: &[u8]) -> Option<Boundary<String>> {
    match effects.transcribe(segment) {
        Err(failure) if failure.is_unavailable() => {
            effects.pause(TRANSCRIBER_RETRY_SECONDS);
            None
        }
        outcome => Some(outcome),
    }
}

pub(super) fn transcribed(effects: &dyn WakeEffects, segment: &[u8]) -> Boundary<String> {
    (1..TRANSCRIBER_ATTEMPTS)
        .find_map(|_| attempt(effects, segment))
        .unwrap_or_else(|| effects.transcribe(segment))
}
