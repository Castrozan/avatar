use super::case::Case;
use super::mock::MockServer;
use avatar_routing::names::vocabulary::{Vocabulary, ranks_from_paths};
use avatar_transcription::scribe::KeytermSource;
use avatar_transcription::{HexClient, ReqwestSender, ScribeClient};
use serde_json::json;
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

pub(crate) type Clients = (Mutex<ScribeClient<ReqwestSender>>, HexClient<ReqwestSender>);

const TEST_KEY: &str = "test-key";

pub(crate) fn clients(
    case: &Case,
    server: &MockServer,
    directory: &Path,
) -> Result<Clients, Box<dyn Error>> {
    let scribe = Mutex::new(scribe_client(case, server, directory)?);
    let discovery = directory.join("local-api.json");
    std::fs::write(
        &discovery,
        json!({"port": server.port, "token": "hex-token"}).to_string(),
    )?;
    Ok((scribe, HexClient::new(ReqwestSender::default(), discovery)))
}

pub(crate) fn scribe_client(
    case: &Case,
    server: &MockServer,
    directory: &Path,
) -> Result<ScribeClient<ReqwestSender>, Box<dyn Error>> {
    let port = endpoint_port(case, server)?;
    let endpoint = format!("http://127.0.0.1:{port}/v1/speech-to-text");
    let source = KeytermSource::new(roster_path(case, directory)?, vocabulary(case, directory)?);
    let key = TEST_KEY.to_owned();
    Ok(ScribeClient::new(
        ReqwestSender::default(),
        endpoint,
        key,
        source,
    ))
}

pub(crate) fn endpoint_port(case: &Case, server: &MockServer) -> Result<u16, Box<dyn Error>> {
    if case.closed_port {
        return closed_port();
    }
    Ok(server.port)
}

pub(crate) fn roster_path(
    case: &Case,
    directory: &Path,
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    if case.roster_missing {
        return Ok(Some(directory.join("missing.txt")));
    }
    let Some(text) = &case.roster_text else {
        return Ok(None);
    };
    let path = directory.join("roster.txt");
    std::fs::write(&path, text)?;
    Ok(Some(path))
}

pub(crate) fn vocabulary(
    case: &Case,
    directory: &Path,
) -> Result<Option<Vocabulary>, Box<dyn Error>> {
    let Some(text) = &case.vocabulary_text else {
        return Ok(None);
    };
    let path = directory.join("words.txt");
    std::fs::write(&path, text)?;
    Ok(ranks_from_paths(path.to_str().unwrap_or_default()))
}

pub(crate) fn closed_port() -> Result<u16, Box<dyn Error>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
