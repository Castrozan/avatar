mod scripted;

use avatar_audio_graph::virtual_microphone::{
    VirtualMicrophoneStart, VirtualMicrophoneStatus, VirtualMicrophoneStop, start, status, stop,
};
use scripted::ScriptedPulse;

const SINK_LOAD: [&str; 4] = [
    "load-module",
    "module-null-sink",
    "sink_name=avatar-virtual-mic",
    "sink_properties=device.description=Avatar Virtual Microphone",
];
const SINK_LISTING: [&str; 3] = ["list", "short", "sinks"];
const MODULE_LISTING: [&str; 3] = ["list", "short", "modules"];
const SOURCE_LISTING: [&str; 3] = ["list", "short", "sources"];
const SINK_LINE: &str = "3\tavatar-virtual-mic\tmodule-null-sink.c";
const MODULE_LINES: &str = "1\tunrelated\tkeep\n9\tmodule-null-sink\tsink_name=avatar-virtual-mic\n12\tmodule-null-sink\tsink_name=avatar-virtual-mic";

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[test]
fn start_loads_the_null_sink_with_the_description() {
    let mut pulse = ScriptedPulse::default().expecting(&SINK_LOAD, Ok("7"));
    assert_eq!(
        start(&mut pulse).map_err(text),
        Ok(VirtualMicrophoneStart::Created)
    );
    assert!(pulse.finished());
}

#[test]
fn start_reports_an_already_present_sink_as_running() {
    let mut pulse = ScriptedPulse::default()
        .expecting(&SINK_LOAD, Err("Module failed"))
        .expecting(&SINK_LISTING, Ok(SINK_LINE));
    assert_eq!(
        start(&mut pulse).map_err(text),
        Ok(VirtualMicrophoneStart::AlreadyRunning)
    );
    assert!(pulse.finished());
}

#[test]
fn start_fails_when_the_load_fails_and_no_sink_is_present() {
    let mut pulse = ScriptedPulse::default()
        .expecting(&SINK_LOAD, Err("Module failed"))
        .expecting(&SINK_LISTING, Ok("1\tAvatarMic\tmodule-null-sink.c"));
    assert_eq!(
        start(&mut pulse).map_err(text),
        Err("Module failed".to_owned())
    );
    assert!(pulse.finished());
}

#[test]
fn stop_unloads_the_first_module_that_names_the_sink() {
    let mut pulse = ScriptedPulse::default()
        .expecting(&MODULE_LISTING, Ok(MODULE_LINES))
        .expecting(&["unload-module", "9"], Ok(""));
    assert_eq!(
        stop(&mut pulse).map_err(text),
        Ok(VirtualMicrophoneStop::Removed)
    );
    assert!(pulse.finished());
}

#[test]
fn stop_reports_not_found_without_unloading_anything() {
    let mut pulse = ScriptedPulse::default().expecting(&MODULE_LISTING, Ok("1\tunrelated\tkeep"));
    assert_eq!(
        stop(&mut pulse).map_err(text),
        Ok(VirtualMicrophoneStop::NotFound)
    );
    assert!(pulse.finished());
}

#[test]
fn status_is_inactive_without_a_sink() {
    let mut pulse = ScriptedPulse::default().expecting(&SINK_LISTING, Ok("1\tAvatarMic\tx"));
    assert_eq!(
        status(&mut pulse).map_err(text),
        Ok(VirtualMicrophoneStatus::Inactive)
    );
    assert!(pulse.finished());
}

#[test]
fn status_lists_the_monitor_source_when_active() {
    let sources = "4\tavatar-virtual-mic.monitor\tx\n5\tother\ty";
    let mut pulse = ScriptedPulse::default()
        .expecting(&SINK_LISTING, Ok(SINK_LINE))
        .expecting(&SOURCE_LISTING, Ok(sources));
    let expected = VirtualMicrophoneStatus::Active {
        monitor_sources: vec!["4\tavatar-virtual-mic.monitor\tx".to_owned()],
    };
    assert_eq!(status(&mut pulse).map_err(text), Ok(expected));
    assert!(pulse.finished());
}
