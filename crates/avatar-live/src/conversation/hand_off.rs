use std::future::Future;

use avatar_floor::addressee::Addressee;
use futures_util::future::OptionFuture;

use super::ending::{Ending, stopped_first};
use super::errand::Errand;
use super::relay::{Podium, Relay};
use crate::boundaries::{AgentAsker, Clock, Console, HolderLookup, VoiceDelivery};
use crate::events::TaskRequest;

pub const WAKE_HANDOVER_SECONDS: f64 = 0.6;
pub const NOBODY_HOLDS_THE_ROOM: &str = "No agent holds the room right now, so nobody can do that.";

fn print_unanswered(screen: &mut impl Console, errand: &Errand, ending: &Ending) {
    if let Some(seconds) = ending.stopped_after() {
        screen.print(&errand.unanswered_line(seconds));
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
    pub async fn hand_off(
        &mut self,
        task: &TaskRequest,
        pace: &mut impl Clock,
        stopped: impl Future<Output = ()>,
    ) {
        pace.sleep(WAKE_HANDOVER_SECONDS).await;
        match self.holders.current() {
            None => self.tell_nobody_holds_the_room(task).await,
            Some(holder) => self.ask_holder(task, holder, pace, stopped).await,
        }
    }

    async fn tell_nobody_holds_the_room(&mut self, task: &TaskRequest) {
        let voice = &mut self.podium.voice;
        voice
            .deliver(&task.task_id, NOBODY_HOLDS_THE_ROOM, true)
            .await;
    }

    async fn ask_holder(
        &mut self,
        task: &TaskRequest,
        holder: Addressee,
        pace: &mut impl Clock,
        stopped: impl Future<Output = ()>,
    ) {
        self.answer_number += 1;
        let errand = Errand::new(task, holder, self.answer_number);
        self.podium.screen.print(&errand.sent_line());
        let ending = self.relay(&errand, pace, stopped).await;
        self.conclude(&errand, &ending).await;
    }

    async fn relay(
        &mut self,
        errand: &Errand,
        pace: &mut impl Clock,
        stopped: impl Future<Output = ()>,
    ) -> Ending {
        let mut relay = Relay::new(pace, &mut self.podium, errand);
        let asking = self.route.ask(&errand.request, &mut relay);
        let stopped = stopped_first(asking, stopped).await;
        relay.ending(stopped)
    }

    async fn conclude(&mut self, errand: &Errand, ending: &Ending) {
        print_unanswered(&mut self.podium.screen, errand, ending);
        let notice = ending.failure_reason();
        OptionFuture::from(notice.map(|reason| self.notify(errand, reason))).await;
    }

    async fn notify(&mut self, errand: &Errand, reason: &str) {
        let delivery = self.route.notify(&errand.request, reason).await;
        self.podium
            .screen
            .print(&errand.notice_line(reason, &delivery));
    }
}
