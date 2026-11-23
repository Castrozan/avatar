use serde_json::json;

use crate::listener::cli::arguments::{EnrollArguments, ForgetArguments, ListenCommand};
use crate::listener::cli::failure::ListenFailure;
use crate::listener::cli::ports::ListenPorts;
use crate::listener::cli::python_json::python_json;
use crate::listener::session::{EMBEDDINGS_FILE, SessionDirectory};
use crate::listener::speaker_identity::enroll as enroll_voice;

fn enroll(
    arguments: &EnrollArguments,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    let path = directory.path().join(EMBEDDINGS_FILE);
    let range = (arguments.first, arguments.last);
    let samples = enroll_voice(ports.voices, &path, &arguments.name, range)?;
    let report = json!({"enrolled": arguments.name, "samples": samples});
    ports.output.line(&python_json(&report));
    Ok(())
}

fn voices(ports: &mut ListenPorts) -> Result<(), ListenFailure> {
    let report = json!({"voices": ports.voices.names()?});
    ports.output.line(&python_json(&report));
    Ok(())
}

fn forget(arguments: &ForgetArguments, ports: &mut ListenPorts) -> Result<(), ListenFailure> {
    ports.voices.forget(&arguments.name)?;
    let report = json!({"forgotten": arguments.name});
    ports.output.line(&python_json(&report));
    Ok(())
}

pub(super) fn run(
    command: &ListenCommand,
    directory: &SessionDirectory,
    ports: &mut ListenPorts,
) -> Result<(), ListenFailure> {
    match command {
        ListenCommand::Enroll(arguments) => enroll(arguments, directory, ports),
        ListenCommand::Voices => voices(ports),
        ListenCommand::Forget(arguments) => forget(arguments, ports),
        _ => Ok(()),
    }
}
