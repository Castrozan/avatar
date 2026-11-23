use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use avatar_listening::barge_in::decision::BargeIn;
use avatar_listening::barge_in::reply_link::control_socket::{CONTROL_SERVER_URL, ReplyLink};
use avatar_listening::listener::capture::{
    NoBargeIn, ProcessCapture, capture_command, command_by_device,
};
use avatar_listening::listener::daemon::{ListenerConfiguration, ListenerKit};
use avatar_listening::listener::onnx::SileroVoiceActivity;
use tokio::runtime::Handle;
use tokio::sync::watch;

use super::hearing::{Hearing, ReplyHandle};
use super::models::{
    SharedTurnModel, TurnJudge, load_turn_model, load_voice_activity, voice_activity_path,
};

type CaptureWords = fn(&str) -> Vec<String>;

fn capture_words(device: &str) -> Vec<String> {
    command_by_device(capture_command)(device)
}

struct BargeInSetup {
    link: Arc<ReplyLink>,
    voice_activity: PathBuf,
    runtime: Handle,
    _stop: watch::Sender<bool>,
}

async fn follow(link: Arc<ReplyLink>) {
    link.follow().await;
}

fn start_link(runtime: &Handle) -> (Arc<ReplyLink>, watch::Sender<bool>) {
    let (stop, stopping) = watch::channel(false);
    let link = ReplyLink::new(CONTROL_SERVER_URL, stopping);
    runtime.spawn(follow(Arc::clone(&link)));
    (link, stop)
}

impl BargeInSetup {
    fn open(runtime: &Handle) -> io::Result<Self> {
        let voice_activity = voice_activity_path()?;
        load_voice_activity(&voice_activity)?;
        let (link, stop) = start_link(runtime);
        Ok(Self {
            link,
            voice_activity,
            runtime: runtime.clone(),
            _stop: stop,
        })
    }

    fn hearing(&self) -> Option<BargeIn<ReplyHandle, SileroVoiceActivity>> {
        let activity = load_voice_activity(&self.voice_activity).ok()?;
        let reply = ReplyHandle::new(Arc::clone(&self.link), self.runtime.clone());
        Some(BargeIn::new(reply, activity))
    }
}

pub(super) struct ProductionKit {
    turn: SharedTurnModel,
    barge_in: Option<BargeInSetup>,
}

impl ProductionKit {
    pub(super) fn load(
        configuration: &ListenerConfiguration,
        runtime: &Handle,
    ) -> io::Result<Self> {
        let turn = load_turn_model()?;
        let barge_in = configuration.barge_in.then(|| BargeInSetup::open(runtime));
        Ok(Self {
            turn,
            barge_in: barge_in.transpose()?,
        })
    }
}

impl ListenerKit for ProductionKit {
    type Launcher = ProcessCapture<CaptureWords>;
    type Judge = TurnJudge;
    type Hearing = Hearing;

    fn launcher(&self) -> Self::Launcher {
        ProcessCapture::new(capture_words as CaptureWords)
    }

    fn judge(&self) -> TurnJudge {
        TurnJudge::new(self.turn.clone())
    }

    fn hearing(&self) -> Hearing {
        let barging = self.barge_in.as_ref().and_then(BargeInSetup::hearing);
        barging.map_or(Hearing::Silent(NoBargeIn), boxed)
    }
}

fn boxed(hearing: BargeIn<ReplyHandle, SileroVoiceActivity>) -> Hearing {
    Hearing::Barging(Box::new(hearing))
}
