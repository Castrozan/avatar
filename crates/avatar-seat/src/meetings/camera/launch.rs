use super::layout::{COLLECTION_NAME, PROFILE_NAME, SCENE_NAME};

const LAUNCH_COMMAND: [&str; 18] = [
    "/usr/bin/open",
    "-g",
    "-n",
    "-a",
    "OBS",
    "--args",
    "--collection",
    COLLECTION_NAME,
    "--profile",
    PROFILE_NAME,
    "--scene",
    SCENE_NAME,
    "--disable-updater",
    "--disable-missing-files-check",
    "--disable-shutdown-check",
    "--use-fake-ui-for-media-stream",
    "--minimize-to-tray",
    "--multi",
];

pub(crate) fn launch_command() -> Vec<String> {
    LAUNCH_COMMAND
        .iter()
        .map(|word| (*word).to_owned())
        .collect()
}
