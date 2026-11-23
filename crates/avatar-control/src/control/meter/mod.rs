mod level;
mod listening;
mod sink;

use std::process::Stdio;
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use listening::{Listening, Outcome};

pub use level::{LevelAccumulator, measure_level};
pub use sink::LevelSink;

const RECORDER_PROGRAM: &str = "parec";
const REPORT_INTERVAL: Duration = Duration::from_millis(33);
const LISTEN_AGAIN: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct MeterSettings {
    pub program: String,
    pub source: String,
    pub listen_again: Duration,
    pub report_every: Duration,
}

impl MeterSettings {
    #[must_use]
    pub fn for_source(source: &str) -> Self {
        Self {
            program: RECORDER_PROGRAM.to_owned(),
            source: source.to_owned(),
            listen_again: LISTEN_AGAIN,
            report_every: REPORT_INTERVAL,
        }
    }
}

pub struct MicrophoneLevelMeter {
    stop: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl MicrophoneLevelMeter {
    pub fn start<Sink: LevelSink>(settings: MeterSettings, sink: Sink) -> Self {
        let (stop, stopped) = watch::channel(false);
        let task = tokio::spawn(listen_until_stopped(settings, sink, stopped));
        Self { stop, task }
    }

    pub async fn stop(self) {
        self.stop.send_replace(true);
        drop(self.task.await);
    }
}

async fn listen_until_stopped<Sink: LevelSink>(
    settings: MeterSettings,
    sink: Sink,
    mut stopped: watch::Receiver<bool>,
) {
    while listen_once(&settings, &sink, &mut stopped).await
        && pause(settings.listen_again, &mut stopped).await
    {}
    sink.send_level(0.0).await;
}

async fn listen_once<Sink: LevelSink>(
    settings: &MeterSettings,
    sink: &Sink,
    stopped: &mut watch::Receiver<bool>,
) -> bool {
    let Some((_recorder, listening)) = start_listening(settings, sink) else {
        return true;
    };
    let outcome = listening.run(stopped).await;
    sink.send_level(0.0).await;
    matches!(outcome, Outcome::RecorderEnded)
}

fn start_listening<Sink: LevelSink>(
    settings: &MeterSettings,
    sink: &Sink,
) -> Option<(Child, Listening<Sink>)> {
    let mut recorder = spawn_recorder(settings)?;
    let listening = Listening::new(&mut recorder, settings.report_every, sink)?;
    Some((recorder, listening))
}

async fn pause(duration: Duration, stopped: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(duration) => true,
        _ = stopped.wait_for(|requested| *requested) => false,
    }
}

const RECORDER_OPTIONS: [&str; 6] = [
    "--property=node.dont-reconnect=true",
    "--raw",
    "--format=float32le",
    "--rate=48000",
    "--channels=1",
    "--latency-msec=20",
];

fn spawn_recorder(settings: &MeterSettings) -> Option<Child> {
    let mut command = Command::new(&settings.program);
    command
        .args(["--device", &settings.source])
        .args(RECORDER_OPTIONS)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    command.spawn().ok()
}
