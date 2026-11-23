use std::time::Duration;

use avatar_live::boundaries::HolderLookup;
use avatar_live::room_voice::RoomVoiceState;
use tokio::time::sleep;

use crate::daemons::live::records::{RoomHolder, VoiceChoice};
use crate::daemons::live::voice::VoiceHandle;

const HOLDER_CHECK: Duration = Duration::from_millis(500);

pub(crate) struct RoomFollower {
    holders: RoomHolder,
    choice: VoiceChoice,
    state: RoomVoiceState,
    voice: VoiceHandle,
}

impl RoomFollower {
    pub(crate) fn new(
        holders: RoomHolder,
        choice: VoiceChoice,
        speaking_voice: &str,
        voice: VoiceHandle,
    ) -> Self {
        Self {
            holders,
            choice,
            state: RoomVoiceState::new(speaking_voice),
            voice,
        }
    }

    fn wanted_voice(&mut self) -> Option<String> {
        let holder = self.holders.current()?;
        Some(self.choice.voice_for_servant(&holder.servant))
    }

    async fn check(&mut self) {
        let wanted = self.wanted_voice();
        let wanted = wanted.filter(|name| self.state.wants_change(Some(name)));
        change_voice(self, wanted).await;
    }
}

async fn change_voice(follower: &mut RoomFollower, wanted: Option<String>) {
    if let Some(voice) = wanted {
        follower.state.change_to(&voice);
        follower.voice.change_to(voice).await;
    }
}

pub(crate) async fn follow_the_room_holder(mut follower: RoomFollower) {
    loop {
        sleep(HOLDER_CHECK).await;
        follower.check().await;
    }
}
