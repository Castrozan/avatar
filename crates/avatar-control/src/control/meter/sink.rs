use std::future::Future;

use crate::server::ServerProbe;

pub trait LevelSink: Clone + Send + Sync + 'static {
    fn send_level(&self, level: f64) -> impl Future<Output = ()> + Send;
}

impl LevelSink for ServerProbe {
    async fn send_level(&self, level: f64) {
        self.voice_level(level).await;
    }
}
