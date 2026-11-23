use std::sync::Arc;

use avatar_conversation::talk::state::seat::microphone_source;
use avatar_live::client::GptLiveClient;
use avatar_live::conversation::Conversation;
use tokio::task::{JoinHandle, spawn_local};

use super::Environment;
use super::launch::{Foundation, Received, Running, guarded, report_microphone};
use crate::daemons::live::devices::PacatSpeaker;
use crate::daemons::live::records::{RunClock, StandardOutput, TranscriptLog};
use crate::daemons::live::room::{
    Hall, HandOffs, Room, RoomFollower, follow_the_room_holder, run_hall,
};
use crate::daemons::live::shutdown::Shutdown;
use crate::daemons::live::voice::{ActorLine, VoiceActor, VoiceHandle, run_actor};

pub(super) struct Launch<'a> {
    environment: &'a Environment,
    shutdown: &'a Arc<Shutdown>,
    foundation: Foundation,
    voice: VoiceHandle,
}

impl<'a> Launch<'a> {
    pub(super) fn new(
        context: (&'a Environment, &'a Arc<Shutdown>),
        foundation: Foundation,
        voice: VoiceHandle,
    ) -> Self {
        Self {
            environment: context.0,
            shutdown: context.1,
            foundation,
            voice,
        }
    }

    pub(super) fn running(
        self,
        client: GptLiveClient,
        speaker: PacatSpeaker,
        line: (ActorLine, Received),
    ) -> Running {
        Running {
            voice: self.voice.clone(),
            actor: self.actor(client, line.0),
            hall: self.hall(speaker, line.1),
            microphone: self.microphone(),
            follower: self.follower(),
        }
    }

    fn actor(&self, client: GptLiveClient, line: ActorLine) -> JoinHandle<()> {
        let shutdown = Arc::clone(self.shutdown);
        let connector = self.foundation.connector.clone();
        let first_voice = &self.foundation.first_voice;
        let actor = VoiceActor::new(client, first_voice, line, connector, Arc::clone(&shutdown));
        spawn_local(guarded(shutdown, run_actor(actor)))
    }

    fn hall(&self, speaker: PacatSpeaker, occurrences: Received) -> JoinHandle<Room> {
        let log = TranscriptLog::new(&self.environment.directory, Arc::clone(self.shutdown));
        let clock = RunClock::new();
        let holders = self.foundation.holders.clone();
        let hand_offs = HandOffs::new(holders, self.voice.clone(), clock);
        let conversation = Conversation::new(speaker, log, StandardOutput);
        let hall = Hall::new(conversation, occurrences, hand_offs, clock);
        spawn_local(guarded(Arc::clone(self.shutdown), run_hall(hall)))
    }

    fn microphone(&self) -> JoinHandle<()> {
        let device = microphone_source(self.foundation.seat).to_owned();
        let (voice, shutdown) = (self.voice.clone(), Arc::clone(self.shutdown));
        spawn_local(report_microphone(device, voice, shutdown))
    }

    fn follower(&self) -> JoinHandle<()> {
        let foundation = &self.foundation;
        let (holders, choice) = (foundation.holders.clone(), foundation.choice.clone());
        let follower =
            RoomFollower::new(holders, choice, &foundation.first_voice, self.voice.clone());
        spawn_local(follow_the_room_holder(follower))
    }
}
