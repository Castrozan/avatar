use std::fs;
use std::path::Path;

use avatar_adapters::commands::{
    Endpoints, HealthProbe, ReadinessTiming, SystemdUnits, TalkListener, TalkProcess,
};
use avatar_conversation::talk::ports::{ListenerSession, ListeningRequest};
use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Map, json};

use crate::tools::{TestResult, Tools};

mod configuration;

pub(crate) const SYSTEMCTL: &str = "echo \"$@\" >> \"$0.log\"; case \"$*\" in \"--user is-active --quiet avatar-live.service\") exit 0;; \"--user is-active\"*) exit 3;; esac";
const CALLS: &str = "--user is-active --quiet avatar-listener.service\n--user is-active --quiet avatar-live.service\n--user stop avatar-live.service\n--user start avatar-listener.service\n";
const RECORDS: [&str; 5] = [
    "transcript.jsonl",
    "embeddings.jsonl",
    "heard",
    "spoken",
    "keep",
];

fn request(purpose: ListenerPurpose) -> ListeningRequest {
    let mut settings = Map::new();
    settings.insert("language".to_owned(), json!("en"));
    settings.insert("barge_in".to_owned(), json!(true));
    let human_source = "AvatarRoomMicrophone".to_owned();
    ListeningRequest {
        purpose,
        human_source,
        settings,
    }
}

pub(super) fn begin(tools: &Tools, directory: &Path, purpose: ListenerPurpose) -> Option<String> {
    let process = TalkProcess::new(tools.programs());
    let health = HealthProbe::new(Endpoints::default());
    let units = SystemdUnits::new(process.clone(), health, ReadinessTiming::default());
    let mut listener = TalkListener::new(directory, process, units);
    let outcome = listener.begin_listening(&request(purpose));
    outcome.err().map(|failure| failure.message().to_owned())
}

fn remaining(directory: &Path) -> Vec<bool> {
    RECORDS
        .iter()
        .map(|name| directory.join(name).exists())
        .collect()
}

#[test]
fn beginning_to_listen_stops_capturing_services_and_starts_the_purpose_unit() -> TestResult {
    let tools = Tools::with(&[("systemctl", SYSTEMCTL)])?;
    let state = tempfile::tempdir()?;
    assert_eq!(
        begin(&tools, state.path(), ListenerPurpose::Conversation),
        None
    );
    assert_eq!(tools.read("systemctl.log"), CALLS);
    Ok(())
}

#[test]
fn only_the_session_records_are_erased() -> TestResult {
    let tools = Tools::with(&[("systemctl", SYSTEMCTL)])?;
    let state = tempfile::tempdir()?;
    RECORDS
        .iter()
        .try_for_each(|name| fs::write(state.path().join(name), "old"))?;
    assert_eq!(begin(&tools, state.path(), ListenerPurpose::Meeting), None);
    assert_eq!(remaining(state.path()), [false, false, false, false, true]);
    Ok(())
}
