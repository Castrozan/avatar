mod commands;
mod fake;
mod host;
mod live;
mod services;
mod turns;

use std::error::Error;
use std::path::Path;

pub(crate) use avatar_conversation::talk::ports::AgentPane;
use avatar_mac::edge::room::{MacRoom, MacRoomSettings};
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::CameraConfig;
use fake::Fake;

pub(crate) fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

pub(crate) fn seat_config(home: &str) -> SeatConfig {
    SeatConfig {
        avatar_host: "chise".to_owned(),
        browser_application: "/Applications/Chromium.app".to_owned(),
        home: home.to_owned(),
        holder_program: String::new(),
    }
}

fn camera_config(home: &str) -> CameraConfig {
    CameraConfig {
        home: home.to_owned(),
        application: "/Applications/OBS.app".to_owned(),
        start_script: "start.lua".to_owned(),
    }
}

fn settings_of(directory: &Path) -> MacRoomSettings {
    let home = directory.display().to_string();
    MacRoomSettings {
        config: seat_config(&home),
        session_directory: directory.join("session"),
        user_identifier: 501,
        camera: camera_config(&home),
        avatar_command: "avatar-command".into(),
        launchctl: directory.join("launchctl"),
    }
}

pub(crate) fn room_with(directory: &Path, fake: Fake) -> Result<MacRoom<Fake>, Box<dyn Error>> {
    Ok(MacRoom::on_this_mac(settings_of(directory))?.with_talk(fake))
}
