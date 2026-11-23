use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::speaking::runtime::channel::{AgentChannel, ChannelRelease};
use crate::speaking::runtime::machine::input::{ChannelOpened, ChannelReleased};
use crate::speaking::runtime::machine::{Input, Runtime};
use crate::speech::floor::{FloorEvent, TurnId};
use crate::synthesis::contract::SpeechGenerator;

async fn open_and_report(
    channel: Arc<dyn AgentChannel>,
    inputs: UnboundedSender<Input>,
    turn: TurnId,
) {
    let result = channel.open().await;
    drop(inputs.send(Input::ChannelOpened(ChannelOpened { turn, result })));
}

async fn release_and_report(
    release: Option<ChannelRelease>,
    inputs: UnboundedSender<Input>,
    turn: TurnId,
) {
    let result = match release {
        Some(release) => release.await,
        None => Ok(()),
    };
    drop(inputs.send(Input::ChannelReleased(ChannelReleased { turn, result })));
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(crate) fn open_channel(&self, turn: TurnId) {
        let channel = Arc::clone(&self.services.channel);
        tokio::spawn(open_and_report(channel, self.inputs.clone(), turn));
    }

    pub(crate) fn channel_opened(&mut self, opened: ChannelOpened) {
        match opened.result {
            Ok(release) => self.keep_release(opened.turn, release),
            Err(failure) => self.fail_turn(opened.turn, &failure.to_string()),
        }
    }

    fn keep_release(&mut self, turn: TurnId, release: ChannelRelease) {
        let mut release = Some(release);
        self.state
            .turns
            .get_mut(&turn)
            .into_iter()
            .for_each(|record| record.progress.release = release.take());
        self.floor_event(FloorEvent::AgentChannelOpened(turn));
    }

    pub(crate) fn fail_turn(&mut self, turn: TurnId, message: &str) {
        self.state
            .turns
            .get_mut(&turn)
            .into_iter()
            .for_each(|record| record.progress.failure = Some(message.to_owned()));
        self.floor_event(FloorEvent::SpeechFailed(turn));
    }

    pub(crate) fn release_channel(&mut self, turn: TurnId) {
        let record = self.state.turns.get_mut(&turn);
        let release = record.and_then(|record| record.progress.release.take());
        tokio::spawn(release_and_report(release, self.inputs.clone(), turn));
    }

    pub(crate) fn channel_released(&mut self, released: ChannelReleased) {
        let failure = released.result.err().map(|failure| failure.to_string());
        self.state
            .turns
            .get_mut(&released.turn)
            .into_iter()
            .for_each(|record| record.override_failure(failure.clone()));
        self.resume(released.turn);
    }
}
