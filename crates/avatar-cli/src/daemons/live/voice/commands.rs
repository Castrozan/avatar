use avatar_live::client::ClientError;
use tokio::task::spawn_local;

use super::actor::VoiceActor;
use super::handle::{
    Change, Delivery, Occurrence, Outbound, Reconnection, Replacement, VoiceCommand,
};

impl VoiceActor {
    pub(super) async fn obey(&mut self, command: VoiceCommand) {
        match command {
            VoiceCommand::Reconnect(Reconnection::Change(change)) => self.begin_replacement(change),
            other => self.settle(other).await,
        }
    }

    async fn settle(&mut self, command: VoiceCommand) {
        let outcome = self.perform(command).await;
        let _ = outcome.map_err(|failure| self.report(&failure.to_string()));
    }

    async fn perform(&mut self, command: VoiceCommand) -> Result<(), ClientError> {
        match command {
            VoiceCommand::Send(outbound) => self.transmit(outbound).await,
            VoiceCommand::Reconnect(step) => self.reconnect(step).await,
        }
    }

    async fn transmit(&mut self, outbound: Outbound) -> Result<(), ClientError> {
        match outbound {
            Outbound::Audio(pcm) => self.client.send_audio(&pcm).await,
            Outbound::Deliver(delivery) => self.deliver(delivery).await,
            Outbound::Close => self.close().await,
        }
    }

    async fn reconnect(&mut self, step: Reconnection) -> Result<(), ClientError> {
        match step {
            Reconnection::Replace(replacement) => self.replace(replacement).await,
            Reconnection::Failed(message) => Err(ClientError(message)),
            Reconnection::Change(_) => Ok(()),
        }
    }

    async fn close(&mut self) -> Result<(), ClientError> {
        self.closing = true;
        self.client.close().await
    }

    async fn deliver(&mut self, delivery: Delivery) -> Result<(), ClientError> {
        self.client.deliver(&delivery.text, delivery.is_final).await
    }

    fn begin_replacement(&self, change: Change) {
        let connector = self.connector.clone();
        spawn_local(connector.replace(change, self.line.own_commands.clone()));
    }

    async fn replace(&mut self, replacement: Replacement) -> Result<(), ClientError> {
        let mut previous = std::mem::replace(&mut self.client, replacement.client);
        previous.close().await?;
        let _ = self.line.occurrences.send(Occurrence::VoiceChanged);
        let _ = replacement.done.send(());
        Ok(())
    }
}
