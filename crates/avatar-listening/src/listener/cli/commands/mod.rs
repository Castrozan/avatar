mod meeting;
mod transcript;

use serde_json::json;

use super::arguments::ListenCommand;
use super::failure::ListenFailure;
use super::ports::ListenPorts;
use super::python_json::python_json;
use super::transcript_wait::last_sequence;
use crate::conversation_addressee::file_store::FileAddresseeStore;
use crate::conversation_addressee::store::AddresseeStore;
use crate::listener::runtime::listening;
use crate::listener::session::{SessionDirectory, stop_listening};

const TRANSCRIPT_FILE: &str = "transcript.jsonl";

pub fn run(
    command: &ListenCommand,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    match command {
        ListenCommand::Start(arguments) => meeting::start(arguments, directory, ports),
        ListenCommand::Stop => stop(ports),
        ListenCommand::Status => status(directory, ports),
        ListenCommand::Addressee => addressee(directory, ports),
        ListenCommand::Read(arguments) => transcript::read(arguments, directory, ports),
        ListenCommand::Follow(arguments) => transcript::follow(arguments, directory, ports),
    }
}

fn stop(ports: &mut ListenPorts) -> Result<(), ListenFailure> {
    stop_listening(&mut ports.units)?;
    ports
        .output
        .line(&python_json(&json!({"listening": false})));
    Ok(())
}

fn status(directory: &SessionDirectory, ports: &mut ListenPorts) -> Result<(), ListenFailure> {
    let report = json!({
        "listening": listening(&mut ports.units),
        "purpose": directory.configured_purpose()?,
        "sources": directory.configured_sources()?,
        "last_sequence": last_sequence(&directory.path().join(TRANSCRIPT_FILE))?,
    });
    ports.output.line(&python_json(&report));
    Ok(())
}

fn addressee(directory: &SessionDirectory, ports: &mut ListenPorts) -> Result<(), ListenFailure> {
    let holder = FileAddresseeStore::new(directory.path()).current()?;
    let report = holder.map(|held| {
        json!({"pane": held.pane, "servant": held.servant, "heard_through": held.heard_through, "host": held.host})
    });
    ports.output.line(&python_json(&report));
    Ok(())
}
