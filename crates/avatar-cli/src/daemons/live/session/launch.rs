use std::io;
use std::sync::Arc;

use avatar_floor::quiet::Seat;
use avatar_live::boundaries::Console;
use avatar_live::client::GptLiveClient;
use avatar_live::session::{MODEL, RATE};
use avatar_live::voices::voice_of_the_room_holder;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::Environment;
use super::seat::configured_seat;
use super::tasks::Launch;
use crate::daemons::live::devices::{PacatSpeaker, sinks_of, stream_microphone};
use crate::daemons::live::records::{RoomHolder, RunClock, StandardOutput, VoiceChoice};
use crate::daemons::live::room::Room;
use crate::daemons::live::shutdown::Shutdown;
use crate::daemons::live::voice::{Connector, Occurrence, VoiceHandle, voice_line};

pub(super) struct Running {
    pub(super) voice: VoiceHandle,
    pub(super) actor: JoinHandle<()>,
    pub(super) hall: JoinHandle<Room>,
    pub(super) microphone: JoinHandle<()>,
    pub(super) follower: JoinHandle<()>,
}

pub(super) struct Foundation {
    pub(super) seat: Seat,
    pub(super) holders: RoomHolder,
    pub(super) choice: VoiceChoice,
    pub(super) connector: Connector,
    pub(super) first_voice: String,
}

pub(super) type Received = mpsc::UnboundedReceiver<Occurrence>;

pub(super) async fn guarded<Output>(
    shutdown: Arc<Shutdown>,
    work: impl Future<Output = Output>,
) -> Output {
    let output = work.await;
    shutdown.request();
    output
}

fn announce(session: Option<String>) {
    let session = session.unwrap_or_else(|| "None".to_owned());
    StandardOutput.print(&format!("{MODEL} live, session {session}"));
}

impl Foundation {
    fn new(environment: &Environment) -> io::Result<Self> {
        Ok(Self {
            seat: configured_seat(&environment.directory)?,
            holders: RoomHolder::new(environment.directory.clone()),
            choice: VoiceChoice::from_environment(),
            connector: Connector::new(&environment.endpoint, &environment.key_file),
            first_voice: String::new(),
        })
    }

    async fn choose_voice(&mut self) {
        let clock = &mut RunClock::new();
        let choice = &self.choice;
        let lookup = &mut self.holders.clone();
        let choose = |name: &str| choice.voice_for_servant(name);
        self.first_voice = voice_of_the_room_holder(lookup, clock, choose).await;
    }

    async fn connect(&mut self) -> io::Result<GptLiveClient> {
        self.choose_voice().await;
        let opened = self.connector.connect(&self.first_voice, false).await;
        let (client, session) = opened.map_err(io::Error::other)?;
        announce(session);
        Ok(client)
    }
}

pub(super) async fn launch(
    environment: &Environment,
    shutdown: &Arc<Shutdown>,
) -> io::Result<Running> {
    let mut foundation = Foundation::new(environment)?;
    let client = foundation.connect().await?;
    let speaker = PacatSpeaker::start(RATE, &sinks_of(foundation.seat)?, Arc::clone(shutdown))?;
    let (voice, line, received) = voice_line();
    let launch = Launch::new((environment, shutdown), foundation, voice);
    Ok(launch.running(client, speaker, (line, received)))
}

pub(super) async fn report_microphone(device: String, voice: VoiceHandle, shutdown: Arc<Shutdown>) {
    if let Err(failure) = stream_microphone(device, voice).await {
        shutdown.fail(failure.to_string());
    }
}
