mod dispatch;
mod polling;

use self::dispatch::{Dispatch, submission_of};
use super::brief::notice_arguments;
use super::report::AgentRequest;
use super::reporter::Reporter;
use crate::boundaries::{AgentAsker, AgentTransport, Courier};

pub struct AgentRoute<Transport: AgentTransport> {
    transport: Transport,
}

impl<Transport: AgentTransport> AgentRoute<Transport> {
    pub fn new(transport: Transport) -> Self {
        Self { transport }
    }

    pub fn into_transport(self) -> Transport {
        self.transport
    }
}

impl<Transport: AgentTransport> AgentAsker for AgentRoute<Transport> {
    async fn ask(&mut self, request: &AgentRequest, courier: &mut impl Courier) {
        let reporter = Reporter::new(courier);
        let dispatch = Dispatch {
            transport: &mut self.transport,
            reporter,
            holder: &request.holder,
        };
        dispatch.ask(&request.question).await;
    }

    async fn notify(&mut self, request: &AgentRequest, reason: &str) -> Result<(), String> {
        let arguments = notice_arguments(&request.holder, &request.question, reason);
        let sent = self.transport.run(&request.holder, &arguments).await;
        submission_of(&sent).map(drop)
    }
}
