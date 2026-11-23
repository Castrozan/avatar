use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{
    ActivityLog, ConsumerLock, Floor, Invocation, ListenerSession, ListeningRequest,
    ProcessFailure, ProcessOutput, Readiness, Subprocess, Units,
};
use avatar_listening::conversation_addressee::Addressee;
use serde_json::{Map, Value};

use super::AgentPane;
use super::fake::{Fake, grant_the_floor};

impl Subprocess for Fake {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure> {
        self.ran.push(invocation.argv.clone());
        let program = invocation.argv.first().cloned().unwrap_or_default();
        grant_the_floor(self, &program);
        let (status, stdout) = self.outputs.get(&program).cloned().unwrap_or_default();
        Ok(ProcessOutput { status, stdout })
    }

    fn relay(&mut self, invocation: &Invocation) -> Result<i32, ProcessFailure> {
        let output = self.run(invocation)?;
        self.out.push_str(&output.stdout);
        Ok(output.status)
    }
}

impl Units for Fake {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.active.contains(unit)
    }

    fn bring_up(&mut self, _target: &str, _checks: &[Readiness]) -> Result<(), TalkFailure> {
        Ok(())
    }

    fn wait_until_ready(
        &mut self,
        _units: &[&str],
        _checks: &[Readiness],
    ) -> Result<(), TalkFailure> {
        Ok(())
    }

    fn remote_seat_listening(&mut self) -> bool {
        false
    }
}

impl ListenerSession for Fake {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure> {
        self.begun.push(request.clone());
        self.files.insert(
            "listener.json".to_owned(),
            r#"{"purpose": "live"}"#.to_owned(),
        );
        Ok(())
    }

    fn erase_session_records(&mut self) -> Result<(), TalkFailure> {
        Ok(())
    }
}

impl Floor for Fake {
    fn give_the_room_to(&mut self, _addressee: &Addressee) -> Result<(), TalkFailure> {
        Ok(())
    }

    fn hand_to_remote_agent(&mut self, _agent: &AgentPane) -> Result<bool, TalkFailure> {
        Ok(false)
    }
}

impl ActivityLog for Fake {
    fn report_talk(&mut self, _action: &str, _pane: Option<&str>) {}

    fn report_speech(&mut self, _pane: Option<&str>, _details: &Map<String, Value>) {}
}

impl ConsumerLock for Fake {
    fn acquire(&mut self) -> Result<(), TalkFailure> {
        Ok(())
    }

    fn release(&mut self) {}
}
