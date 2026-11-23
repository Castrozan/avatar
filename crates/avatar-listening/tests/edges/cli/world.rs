use std::collections::BTreeSet;
use std::io;
use std::path::Path;

use crate::edges::helpers::{Failure, strings};
use crate::support::text;
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use avatar_listening::listener::cli::ListenFailure;
use serde_json::Value;

pub(super) struct World {
    pub(super) active: Vec<String>,
    pub(super) sources: BTreeSet<String>,
    pub(super) default_mic: String,
    transcript: Option<String>,
    holder: bool,
    configuration: Option<String>,
}

pub(super) struct Report {
    pub(super) lines: Vec<String>,
    pub(super) failure: Option<ListenFailure>,
    pub(super) events: Vec<Vec<String>>,
    pub(super) configuration: Option<Value>,
}

impl World {
    pub(super) fn of(case: &Value) -> Result<Self, Failure> {
        Ok(Self {
            active: strings(&case["active"])?,
            sources: strings(&case["sources"])?.into_iter().collect(),
            default_mic: text(&case["default_mic"])?.to_owned(),
            transcript: case["transcript_text"].as_str().map(str::to_owned),
            holder: case["holder"] == true,
            configuration: case["existing_config"]
                .as_object()
                .map(|_| case["existing_config"].to_string()),
        })
    }

    pub(super) fn populate(&self, directory: &Path) -> Result<(), Failure> {
        write_present(directory, "transcript.jsonl", self.transcript.as_ref())?;
        write_present(directory, "listener.json", self.configuration.as_ref())?;
        self.holder.then(|| hand_holder(directory)).transpose()?;
        Ok(())
    }
}

fn write_present(directory: &Path, name: &str, content: Option<&String>) -> Result<(), Failure> {
    let write = |body: &String| std::fs::write(directory.join(name), body);
    content.map(write).transpose()?;
    Ok(())
}

fn hand_holder(directory: &Path) -> io::Result<()> {
    let host = Some("kira".to_owned());
    let holder = Addressee {
        pane: "w1:p2".into(),
        servant: "Voyager".into(),
        heard_through: 4,
        host,
    };
    FileAddresseeStore::new(directory).hand_to(&holder)
}

pub(super) fn read_configuration(directory: &Path) -> Result<Option<Value>, Failure> {
    let bytes = std::fs::read(directory.join("listener.json")).ok();
    Ok(bytes
        .map(|content| serde_json::from_slice(&content))
        .transpose()?)
}
