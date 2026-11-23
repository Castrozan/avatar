use std::future::Future;
use std::time::Duration;

use serde_json::{Map, Value};

use super::READ_TIMEOUT_SECONDS;
use super::gain::{Sink, levels_from_sinks};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkReadFailure;

pub trait LevelSource: Send + Sync + 'static {
    fn read_sinks(&self) -> impl Future<Output = Result<Vec<Sink>, SinkReadFailure>> + Send;
}

pub async fn current<Source: LevelSource>(source: &Source) -> Map<String, Value> {
    let limit = Duration::from_secs(READ_TIMEOUT_SECONDS);
    match tokio::time::timeout(limit, source.read_sinks()).await {
        Ok(Ok(sinks)) => levels_from_sinks(&sinks),
        _ => Map::new(),
    }
}
