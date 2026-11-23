use avatar_floor::addressee::Addressee;

use super::relay::{Podium, Relay};
use crate::agent::report::{AgentQuestion, AgentRequest};
use crate::boundaries::{AgentAsker, Clock, Console, HolderLookup, VoiceDelivery};
use crate::events::TaskRequest;

pub const WAKE_HANDOVER_SECONDS: f64 = 0.6;
pub const NOBODY_HOLDS_THE_ROOM: &str = "No agent holds the room right now, so nobody can do that.";

fn task_line(task: &TaskRequest, holder: &Addressee) -> String {
    format!(
        "task for {} at {}: {}",
        holder.servant, holder.pane, task.request
    )
}

fn question_of(task: &TaskRequest, number: u32) -> AgentQuestion {
    AgentQuestion {
        request: task.request.clone(),
        number,
    }
}

pub struct HandOffDesk<
    Holders: HolderLookup,
    Asker: AgentAsker,
    Voice: VoiceDelivery,
    Screen: Console,
> {
    pub holders: Holders,
    pub route: Asker,
    pub podium: Podium<Voice, Screen>,
    pub answer_number: u32,
}

impl<Holders: HolderLookup, Asker: AgentAsker, Voice: VoiceDelivery, Screen: Console>
    HandOffDesk<Holders, Asker, Voice, Screen>
{
    pub async fn hand_off(&mut self, task: &TaskRequest, pace: &mut impl Clock) {
        pace.sleep(WAKE_HANDOVER_SECONDS).await;
        match self.holders.current() {
            None => self.tell_nobody_holds_the_room(task).await,
            Some(holder) => self.ask_holder(task, holder, pace).await,
        }
    }

    async fn tell_nobody_holds_the_room(&mut self, task: &TaskRequest) {
        let voice = &mut self.podium.voice;
        voice
            .deliver(&task.task_id, NOBODY_HOLDS_THE_ROOM, true)
            .await;
    }

    async fn ask_holder(&mut self, task: &TaskRequest, holder: Addressee, pace: &mut impl Clock) {
        self.podium.screen.print(&task_line(task, &holder));
        self.answer_number += 1;
        let question = question_of(task, self.answer_number);
        let mut relay = Relay::new(pace, &mut self.podium, task, &holder.servant);
        let request = AgentRequest { holder, question };
        self.route.ask(&request, &mut relay).await;
    }
}
