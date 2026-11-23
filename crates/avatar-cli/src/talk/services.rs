use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::{
    AgentPane, ConsumerLock, Floor, ListenerSession, ListeningRequest, Readiness, Units,
};
use avatar_listening::conversation_addressee::Addressee;

use super::boundaries::TalkBoundaries;

impl<Booted: Units, Listening> Units for TalkBoundaries<Booted, Listening> {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.units.unit_active(unit)
    }

    fn bring_up(&mut self, target: &str, checks: &[Readiness]) -> Result<(), TalkFailure> {
        self.units.bring_up(target, checks)
    }

    fn wait_until_ready(
        &mut self,
        units: &[&str],
        checks: &[Readiness],
    ) -> Result<(), TalkFailure> {
        self.units.wait_until_ready(units, checks)
    }

    fn remote_seat_listening(&mut self) -> bool {
        self.units.remote_seat_listening()
    }
}

impl<Booted, Listening: ListenerSession> ListenerSession for TalkBoundaries<Booted, Listening> {
    fn begin_listening(&mut self, request: &ListeningRequest) -> Result<(), TalkFailure> {
        self.listener.begin_listening(request)
    }

    fn erase_session_records(&mut self) -> Result<(), TalkFailure> {
        self.listener.erase_session_records()
    }
}

impl<Booted, Listening> Floor for TalkBoundaries<Booted, Listening> {
    fn give_the_room_to(&mut self, addressee: &Addressee) -> Result<(), TalkFailure> {
        self.floor.give_the_room_to(addressee)
    }

    fn hand_to_remote_agent(&mut self, agent: &AgentPane) -> Result<bool, TalkFailure> {
        self.floor.hand_to_remote_agent(agent)
    }
}

impl<Booted, Listening> ConsumerLock for TalkBoundaries<Booted, Listening> {
    fn acquire(&mut self) -> Result<(), TalkFailure> {
        self.lock.acquire()
    }

    fn release(&mut self) {
        self.lock.release();
    }
}
