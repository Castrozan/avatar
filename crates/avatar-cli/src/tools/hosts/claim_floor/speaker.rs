use std::io;
use std::path::Path;
use std::process::ExitCode;

use avatar_floor::addressee::current;
use avatar_floor::floor::RemoteAgent;
use avatar_wake::floor_cli::speaker_claim_outcome;

use super::lookup::servant_at;
use super::{conclude, unreadable_session};

pub(super) fn claim_for_speaker(directory: &Path, speaker: &RemoteAgent) -> io::Result<ExitCode> {
    match servant_at(speaker) {
        Ok(servant) => {
            let holder = current(directory)?;
            conclude(speaker_claim_outcome(holder.as_ref(), speaker, servant))
        }
        Err(reason) => unreadable_session(&reason),
    }
}
