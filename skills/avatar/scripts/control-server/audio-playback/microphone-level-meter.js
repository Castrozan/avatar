const { spawn } = require("node:child_process");

function measureLevel(samples) {
  let power = 0;
  const count = Math.floor(samples.length / 4);
  for (let index = 0; index < count; index++) {
    const value = samples.readFloatLE(index * 4);
    if (Number.isFinite(value)) power += value * value;
  }
  return count ? Math.sqrt(power / count) : 0;
}

const LISTEN_AGAIN_MILLISECONDS = 1000;

function spawnRecorder(sourceName) {
  return spawn(
    "parec",
    [
      "--device",
      sourceName,
      "--property=node.dont-reconnect=true",
      "--raw",
      "--format=float32le",
      "--rate=48000",
      "--channels=1",
      "--latency-msec=20",
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
}

function hearLevels(recorder, reportLevel) {
  let pending = Buffer.alloc(0);
  recorder.stdout.on("data", (chunk) => {
    const samples = Buffer.concat([pending, chunk]);
    const alignedSize = samples.length - (samples.length % 4);
    reportLevel(measureLevel(samples.subarray(0, alignedSize)));
    pending = Buffer.from(samples.subarray(alignedSize));
  });
  recorder.stderr.on("data", (chunk) => console.error(chunk.toString().trim()));
  recorder.on("error", (error) => console.error(error.message));
}

function startMicrophoneLevelMeter(
  clientRegistry,
  sourceName,
  listenAgainMilliseconds = LISTEN_AGAIN_MILLISECONDS,
) {
  let level = 0;
  let stopped = false;
  let recorder;
  let listeningAgain;
  const listen = () => {
    recorder = spawnRecorder(sourceName);
    hearLevels(recorder, (heard) => {
      level = heard;
    });
    recorder.on("close", () => {
      level = 0;
      if (!stopped)
        listeningAgain = setTimeout(listen, listenAgainMilliseconds);
    });
  };
  listen();
  const interval = setInterval(() => {
    clientRegistry.sendVoiceLevel(level);
    level = 0;
  }, 33);
  return () => {
    stopped = true;
    clearTimeout(listeningAgain);
    clearInterval(interval);
    clientRegistry.sendVoiceLevel(0);
    recorder.kill("SIGTERM");
  };
}

module.exports = { measureLevel, startMicrophoneLevelMeter };
