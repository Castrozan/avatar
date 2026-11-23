use crate::speaking::runtime::platform::Platform;
use crate::speaking::{OutputRoute, SinkRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackTarget {
    LipSync,
    Mic,
    Speakers,
    Discord,
    Seat,
}

#[must_use]
pub fn playback_targets(route: &OutputRoute, platform: Platform) -> Vec<PlaybackTarget> {
    match (route, platform) {
        (OutputRoute::Discord, _) => vec![PlaybackTarget::Discord],
        (OutputRoute::Sinks(_), Platform::Darwin) => vec![PlaybackTarget::Seat],
        (OutputRoute::Sinks(roles), Platform::Linux) => {
            roles.iter().copied().map(target_of).collect()
        }
    }
}

fn target_of(role: SinkRole) -> PlaybackTarget {
    match role {
        SinkRole::LipSync => PlaybackTarget::LipSync,
        SinkRole::Mic => PlaybackTarget::Mic,
        SinkRole::Speakers => PlaybackTarget::Speakers,
    }
}
