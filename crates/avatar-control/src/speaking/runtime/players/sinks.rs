use std::path::Path;
use std::sync::Arc;

use super::{PlaybackTarget, Players, RunningPlayback};
use crate::playback::{AudioPlayer, Playback, ProgramAudioPlayer, PulseSinkPlayer};

const LIP_SYNC_SINK: &str = "AvatarLipSync";
const MIC_SINK: &str = "AvatarMic";
const SPEAKER_SINK: &str = "default";

#[derive(Debug, Clone)]
pub struct PlayerPrograms {
    pub paplay: String,
    pub discord_voice: String,
    pub seat_play: String,
}

impl Default for PlayerPrograms {
    fn default() -> Self {
        Self {
            paplay: "paplay".to_owned(),
            discord_voice: "discord-voice".to_owned(),
            seat_play: "avatar-seat-play".to_owned(),
        }
    }
}

pub struct SinkPlayers {
    lip_sync: PulseSinkPlayer,
    mic: PulseSinkPlayer,
    speakers: PulseSinkPlayer,
    discord: ProgramAudioPlayer,
    seat: ProgramAudioPlayer,
}

impl SinkPlayers {
    #[must_use]
    pub fn new(programs: &PlayerPrograms, runtime_directory: &str) -> Self {
        let paplay = programs.paplay.as_str();
        Self {
            lip_sync: PulseSinkPlayer::only_sink(paplay, LIP_SYNC_SINK, runtime_directory),
            mic: PulseSinkPlayer::only_sink(paplay, MIC_SINK, runtime_directory),
            speakers: PulseSinkPlayer::new(paplay, SPEAKER_SINK, runtime_directory),
            discord: ProgramAudioPlayer::new(&programs.discord_voice, vec!["play".to_owned()]),
            seat: ProgramAudioPlayer::new(&programs.seat_play, Vec::new()),
        }
    }

    fn player(&self, target: PlaybackTarget) -> &dyn AudioPlayer {
        match target {
            PlaybackTarget::LipSync => &self.lip_sync,
            PlaybackTarget::Mic => &self.mic,
            PlaybackTarget::Speakers => &self.speakers,
            PlaybackTarget::Discord => &self.discord,
            PlaybackTarget::Seat => &self.seat,
        }
    }
}

impl Players for SinkPlayers {
    fn start(&self, target: PlaybackTarget, audio_path: &Path) -> RunningPlayback {
        running(self.player(target).play(audio_path))
    }
}

fn running(playback: Playback) -> RunningPlayback {
    let handle = playback.stop_handle();
    RunningPlayback {
        stop: Arc::new(move || handle.stop()),
        finished: Box::pin(async move {
            drop(playback.finished().await);
        }),
    }
}
