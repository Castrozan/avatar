use std::io;

use avatar_listening::user_units::Clock;
use avatar_listening::user_units::launchd::LaunchctlPrint;
use avatar_listening::user_units::launchd::effects::LaunchctlControl;

pub(super) struct ScriptedLaunchctl {
    pub(super) prints: Vec<String>,
    pub(super) calls: Vec<Vec<String>>,
}

#[derive(Default)]
pub(super) struct ArithmeticClock {
    reads: u32,
    pub(super) sleeps: usize,
}

impl LaunchctlPrint for ScriptedLaunchctl {
    fn printed(&mut self, agent_target: &str) -> String {
        self.calls.push(vec!["print".into(), agent_target.into()]);
        let queue = &mut self.prints;
        let answer = queue.first().cloned().unwrap_or_default();
        (queue.len() > 1).then(|| queue.remove(0));
        answer
    }
}

impl LaunchctlControl for ScriptedLaunchctl {
    fn kickstart(&mut self, target: &str) -> io::Result<()> {
        self.calls.push(vec!["kickstart".into(), target.into()]);
        Ok(())
    }

    fn terminate(&mut self, target: &str) {
        self.calls
            .push(vec!["kill".into(), "TERM".into(), target.into()]);
    }
}

impl Clock for ArithmeticClock {
    fn monotonic_seconds(&mut self) -> f64 {
        self.reads += 1;
        100.0 + f64::from(self.reads - 1) * 3.0
    }

    fn sleep_seconds(&mut self, _seconds: f64) {
        self.sleeps += 1;
    }
}
