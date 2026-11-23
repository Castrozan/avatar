use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::server::command::Command;
use crate::session::HealthSnapshot;

#[derive(Clone)]
pub struct ServerProbe {
    commands: Sender<Command>,
}

impl ServerProbe {
    pub(crate) fn new(commands: Sender<Command>) -> Self {
        Self { commands }
    }

    pub async fn health(&self) -> Option<HealthSnapshot> {
        let (answer, answered) = oneshot::channel();
        self.commands.send(Command::Health(answer)).await.ok()?;
        answered.await.ok()
    }

    pub async fn voice_level(&self, level: f64) {
        drop(self.commands.send(Command::VoiceLevel(level)).await);
    }
}
