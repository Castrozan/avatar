use super::case::{Actual, Call, Case, Operation, Replay, decode_hex};
use super::mock::MockServer;
use super::setup::clients;
use avatar_transcription::{
    HexClient, ReqwestSender, ScribeClient, TranscriptionError, transcribe_with_local_fallback,
};
use futures_util::stream::{self, StreamExt};
use serde_json::{Value, json};
use std::error::Error;
use tempfile::TempDir;
use tokio::sync::Mutex;

type Answer = Result<Value, TranscriptionError>;

pub(crate) struct Environment {
    server: MockServer,
    scribe: Mutex<ScribeClient<ReqwestSender>>,
    hex: HexClient<ReqwestSender>,
    _directory: TempDir,
}

impl Environment {
    async fn start(case: &Case) -> Result<Self, Box<dyn Error>> {
        let server = MockServer::start(case.responses.clone()).await?;
        let directory = TempDir::new()?;
        Self::assemble(case, server, directory)
    }

    fn assemble(
        case: &Case,
        server: MockServer,
        directory: TempDir,
    ) -> Result<Self, Box<dyn Error>> {
        let (scribe, hex) = clients(case, &server, directory.path())?;
        Ok(Self {
            server,
            scribe,
            hex,
            _directory: directory,
        })
    }

    async fn run_calls(&self, case: &Case) -> Vec<Actual> {
        let pcm = decode_hex(&case.pcm_hex);
        stream::iter(&case.calls)
            .then(|call| self.run_call(call, &pcm))
            .collect()
            .await
    }

    async fn run_call(&self, call: &Call, pcm: &[u8]) -> Actual {
        let mut diagnostics = Vec::new();
        let value = self.perform(call, pcm, &mut diagnostics).await;
        (value, diagnostics)
    }

    async fn perform(&self, call: &Call, pcm: &[u8], diagnostics: &mut Vec<String>) -> Answer {
        match call.op {
            Operation::Cloud | Operation::Fallback => self.remote(call, pcm, diagnostics).await,
            Operation::Hex | Operation::Keyterms => self.offline(call, pcm).await,
        }
    }

    async fn remote(&self, call: &Call, pcm: &[u8], diagnostics: &mut Vec<String>) -> Answer {
        match call.op {
            Operation::Fallback => self.fallback(call, pcm, diagnostics).await,
            _ => self.cloud(call, pcm, diagnostics).await,
        }
    }

    async fn offline(&self, call: &Call, pcm: &[u8]) -> Answer {
        match call.op {
            Operation::Keyterms => self.keyterms().await,
            _ => self.local(call, pcm).await,
        }
    }

    async fn cloud(&self, call: &Call, pcm: &[u8], diagnostics: &mut Vec<String>) -> Answer {
        let mut scribe = self.scribe.lock().await;
        scribe
            .transcribe(pcm, call.language.as_deref(), diagnostics)
            .await
            .map(Value::String)
    }

    async fn fallback(&self, call: &Call, pcm: &[u8], diagnostics: &mut Vec<String>) -> Answer {
        let mut scribe = self.scribe.lock().await;
        let language = call.language.as_deref();
        transcribe_with_local_fallback(&mut scribe, &self.hex, pcm, language, diagnostics)
            .await
            .map(Value::String)
    }

    async fn keyterms(&self) -> Answer {
        Ok(json!(self.scribe.lock().await.keyterms()))
    }

    async fn local(&self, call: &Call, pcm: &[u8]) -> Answer {
        self.hex
            .transcribe(pcm, call.language.as_deref())
            .await
            .map(Value::String)
    }
}

pub(crate) async fn replay(case: &Case) -> Result<Replay, Box<dyn Error>> {
    let environment = Environment::start(case).await?;
    let outcomes = environment.run_calls(case).await;
    Ok(Replay {
        outcomes,
        requests: environment.server.recorded().await,
        remaining: environment.server.remaining().await,
    })
}
