use crate::fakes::executable;
use avatar_listening::conversation_state::ListenerPurpose;
use avatar_mac::edge::room::{MacRoom, MacRoomSettings};
use avatar_mac::session::ListeningPlan;
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::CameraConfig;
use std::path::Path;

pub(crate) fn launchctl(directory: &Path) -> std::io::Result<std::path::PathBuf> {
    let running = directory.join("running");
    let body = format!(
        "printf '%s\\n' \"$*\" >> {log}\ncase \"$1\" in\nprint) [ -f {running} ] && printf '\\tstate = running\\n' ;;\nkickstart) touch {running} ;;\nkill) rm -f {running} ;;\nesac",
        log = directory.join("log").display(),
        running = running.display(),
    );
    executable(directory, "launchctl", &body)
}

pub(crate) fn audio_switcher(directory: &Path) -> std::io::Result<std::path::PathBuf> {
    let body = format!(
        "printf '%s\\n' \"$*\" >> {log}\ncase \"$1\" in\n-c) printf '{{\"name\": \"Headset\", \"type\": \"%s\", \"id\": \"7\", \"uid\": \"%s\"}}\\n' \"$3\" \"$(cat {directory}/default-$3 2>/dev/null || printf headset-$3)\" ;;\n-t) printf '%s' \"$4\" > {directory}/default-$2 ;;\nesac",
        log = directory.join("switches").display(),
        directory = directory.display(),
    );
    executable(directory, "SwitchAudioSource", &body)
}

pub(crate) fn config_of(directory: &Path) -> SeatConfig {
    SeatConfig {
        avatar_host: "chise".to_owned(),
        browser_application: "/Applications/Chromium.app".to_owned(),
        home: directory.display().to_string(),
        holder_program: String::new(),
    }
}

pub(crate) fn camera_of(config: &SeatConfig) -> CameraConfig {
    CameraConfig {
        home: config.home.clone(),
        application: "/Applications/OBS.app".to_owned(),
        start_script: "start.lua".to_owned(),
    }
}

pub(crate) fn settings_of(directory: &Path) -> std::io::Result<MacRoomSettings> {
    let config = config_of(directory);
    Ok(MacRoomSettings {
        camera: camera_of(&config),
        config,
        session_directory: directory.join("session"),
        user_identifier: 501,
        avatar_command: "avatar-command".into(),
        launchctl: launchctl(directory)?,
        audio_switcher: audio_switcher(directory)?,
    })
}

pub(crate) fn room(directory: &Path) -> TestResult2<MacRoom> {
    Ok(MacRoom::on_this_mac(settings_of(directory)?)?)
}

pub(crate) type TestResult2<Value> = Result<Value, Box<dyn std::error::Error>>;

pub(super) fn meeting_plan() -> ListeningPlan {
    ListeningPlan {
        purpose: ListenerPurpose::Meeting,
        sources: vec![("meeting".to_owned(), "meeting".to_owned())],
        language: Some("pt".to_owned()),
        identify_speakers: true,
        barge_in: false,
    }
}
