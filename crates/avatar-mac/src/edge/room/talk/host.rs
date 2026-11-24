use avatar_conversation::talk::cli::{
    forwarded_from_this_machine, forwarded_to_the_conversation_opened_elsewhere,
};
use avatar_conversation::talk::ports::{
    Console, Environment as ProcessEnvironment, Files, Subprocess, Units,
};
use avatar_conversation::talk::state::elsewhere::remember_the_conversation_at;
use avatar_seat::edges::{HttpProbe, SystemClock, SystemProcesses};
use avatar_seat::local::{LocalSeat, SEAT_PORT, SeatConfig};
use avatar_seat::talk::{Environment, RemoteTalk};

use crate::edge::failure::edge_failure;
use crate::failure::Failure;
use crate::talk::HostTalk;

pub struct MacHostTalk<Boundary> {
    pub boundary: Boundary,
    pub config: SeatConfig,
}

impl<Boundary> HostTalk for MacHostTalk<Boundary>
where
    Boundary: Subprocess + ProcessEnvironment + Console + Files + Units,
{
    fn forwarded_from_this_machine(&mut self, arguments: &[String]) -> Option<i32> {
        forwarded_from_this_machine(&mut self.boundary, arguments)
    }

    fn forwarded_to_the_conversation_opened_elsewhere(
        &mut self,
        arguments: &[String],
    ) -> Option<i32> {
        forwarded_to_the_conversation_opened_elsewhere(&mut self.boundary, arguments)
    }

    fn remember_the_conversation_at_the_host(&mut self) {
        remember_the_conversation_at(&mut self.boundary, &self.config.avatar_host);
    }

    fn with_the_agent_that_receives_live_tasks(&mut self, arguments: &[String]) -> Vec<String> {
        arguments.to_vec()
    }

    fn carry_out_at_host(&mut self, arguments: &[String]) -> Result<i32, Failure> {
        let environment: Environment = std::env::vars().collect();
        let clock = SystemClock::new();
        let probe = HttpProbe { port: SEAT_PORT };
        let local = LocalSeat::new(&self.config, &SystemProcesses, &clock, &probe);
        let report = RemoteTalk::new(local, &environment)
            .run(arguments)
            .map_err(|error| edge_failure(&error))?;
        self.boundary.print_out(&report.stdout);
        self.boundary.print_err(&report.stderr);
        Ok(report.code)
    }
}
