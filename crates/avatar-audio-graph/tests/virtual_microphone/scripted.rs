use std::collections::VecDeque;

use avatar_audio_graph::pulse::control::{PulseControl, PulseError};

type Reply = Result<&'static str, &'static str>;

#[derive(Default)]
pub(crate) struct ScriptedPulse {
    pub(crate) expected: VecDeque<(Vec<&'static str>, Reply)>,
    pub(crate) mismatches: Vec<String>,
}

impl ScriptedPulse {
    pub(crate) fn expecting(mut self, argv: &[&'static str], reply: Reply) -> Self {
        self.expected.push_back((argv.to_vec(), reply));
        self
    }

    pub(crate) fn finished(&self) -> bool {
        self.expected.is_empty() && self.mismatches.is_empty()
    }
}

impl PulseControl for ScriptedPulse {
    fn run(&mut self, arguments: &[&str]) -> Result<String, PulseError> {
        answer(self, arguments)
    }
}

pub(crate) fn answer(pulse: &mut ScriptedPulse, arguments: &[&str]) -> Result<String, PulseError> {
    let Some((argv, reply)) = pulse.expected.pop_front() else {
        pulse.mismatches.push(format!("unexpected {arguments:?}"));
        return Err(PulseError::new("unexpected call"));
    };
    if arguments != argv.as_slice() {
        pulse
            .mismatches
            .push(format!("got {arguments:?} expected {argv:?}"));
    }
    reply.map(str::to_owned).map_err(PulseError::new)
}
