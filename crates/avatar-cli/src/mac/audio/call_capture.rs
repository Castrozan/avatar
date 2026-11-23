use std::io;
use std::process::ExitCode;

use avatar_mac::client::{call_capture_exit, copy_call_audio};
use avatar_mac::edge::client::CallFrames;

pub(super) fn run() -> ExitCode {
    let Ok(mut frames) = CallFrames::connect_on_this_mac() else {
        return ExitCode::FAILURE;
    };
    let outcome = copy_call_audio(frames.by_ref(), &mut io::stdout().lock());
    let failed = call_capture_exit(&outcome) != 0 || frames.ended_abnormally();
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
