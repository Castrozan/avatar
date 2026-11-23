const { spawn } = require("child_process");

const PLAYBACK_CLIENT_NAME = "avatar-speech";
const NEVER_FALL_BACK_TO_ANOTHER_SINK = "--property=node.dont-fallback=true";

function buildPulseAudioEnvironment() {
  return {
    ...process.env,
    XDG_RUNTIME_DIR: `/run/user/${process.getuid()}`,
  };
}

function playAudioFile(audioPath, sinkName, streamProperties) {
  const player = spawn(
    "paplay",
    [
      "--client-name",
      PLAYBACK_CLIENT_NAME,
      "--device",
      sinkName,
      ...streamProperties,
      audioPath,
    ],
    { stdio: ["ignore", "ignore", "pipe"], env: buildPulseAudioEnvironment() },
  );

  player.stderr.on("data", (chunk) => {
    console.error(`🔊 paplay error (${sinkName}): ${chunk.toString().trim()}`);
  });

  return {
    finished: new Promise((resolve) => {
      player.on("close", resolve);
      player.on("error", resolve);
    }),
    stop: () => player.kill("SIGTERM"),
  };
}

function playAudioFileToSink(audioPath, sinkName) {
  return playAudioFile(audioPath, sinkName, []);
}

function playAudioFileOnlyToSink(audioPath, sinkName) {
  return playAudioFile(audioPath, sinkName, [NEVER_FALL_BACK_TO_ANOTHER_SINK]);
}

module.exports = { playAudioFileOnlyToSink, playAudioFileToSink };
