#[derive(Debug, Clone, Copy)]
pub(super) enum Direction {
    Input,
    Output,
}

pub(super) const DIRECTIONS: [Direction; 2] = [Direction::Input, Direction::Output];

impl Direction {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Output => "output",
        }
    }

    pub(super) const fn call_device(self) -> &'static str {
        match self {
            Self::Input => "BlackHole2ch_UID",
            Self::Output => "BlackHole16ch_UID",
        }
    }

    pub(super) const fn built_in_device(self) -> &'static str {
        match self {
            Self::Input => "BuiltInMicrophoneDevice",
            Self::Output => "BuiltInSpeakerDevice",
        }
    }
}
