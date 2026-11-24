import { chosenMicrophone, openMicrophoneOn } from "/seat-microphone.js";

const MICROPHONE_RATE = 16000;
const WORKLETS = "/seat-audio-worklets.js";
const state = document.getElementById("state");

function stopTracks(stream) {
  for (const track of stream.getTracks()) track.stop();
}

async function openAudioGraph() {
  const context = new AudioContext({ sampleRate: MICROPHONE_RATE });
  await context.audioWorklet.addModule(WORKLETS);
  await context.resume();
  return context;
}

function pinnedChoice(previous) {
  return chosenMicrophone(navigator.mediaDevices, { pinned: true, previous });
}

async function openPinned(deviceId) {
  const stream = await openMicrophoneOn(navigator.mediaDevices, deviceId);
  return { deviceId, stream };
}

function followTheChosenMicrophone(capture, frames, first) {
  let current = first;
  let source = capture.createMediaStreamSource(current.stream);
  source.connect(frames);
  let switching = Promise.resolve();
  const switchWhenTheChoiceMoves = async () => {
    const deviceId = await pinnedChoice(current.deviceId);
    if (deviceId === current.deviceId) return;
    const next = await openPinned(deviceId);
    source.disconnect();
    stopTracks(current.stream);
    current = next;
    source = capture.createMediaStreamSource(current.stream);
    source.connect(frames);
  };
  navigator.mediaDevices.addEventListener("devicechange", () => {
    switching = switching.then(switchWhenTheChoiceMoves).catch((error) => {
      state.textContent = `${error.name}: ${error.message}`;
    });
  });
  return () => current.stream;
}

async function captureForTheWake() {
  const first = await openPinned(await pinnedChoice(null));
  const capture = await openAudioGraph();
  const frames = new AudioWorkletNode(capture, "microphone-frames");
  const socket = new WebSocket(`ws://${location.host}/audio`);
  socket.binaryType = "arraybuffer";
  const microphone = followTheChosenMicrophone(capture, frames, first);
  frames.port.onmessage = (event) => {
    if (socket.readyState === WebSocket.OPEN) socket.send(event.data);
  };
  socket.onopen = () => {
    state.textContent = "The wake listener hears through this page";
  };
  socket.onclose = () => {
    stopTracks(microphone());
    capture.close();
    window.close();
  };
}

captureForTheWake().catch((error) => {
  const failure = `${error.name}: ${error.message}`;
  state.textContent = failure;
  fetch(`/failure?${encodeURIComponent(failure)}`);
});
