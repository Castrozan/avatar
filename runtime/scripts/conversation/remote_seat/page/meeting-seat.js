import {
  builtInId,
  currentDevice,
  devicesOfKind,
  isCallDevice,
  realDeviceId,
} from "/microphone-choice.js";

const CALL_MICROPHONE_OUTPUT = "BlackHole 2ch";
const CALL_SPEAKER_INPUT = "BlackHole 16ch";
const CALL_RATE = 16000;
const SYSTEM_DEFAULT_DEVICES = ["default", "communications"];

export function findDevice(devices, kind, name) {
  return devices.find(
    (device) =>
      device.kind === kind &&
      !SYSTEM_DEFAULT_DEVICES.includes(device.deviceId) &&
      device.label.includes(name),
  );
}

export function requireCallDevices(devices) {
  const callMicrophone = findDevice(
    devices,
    "audiooutput",
    CALL_MICROPHONE_OUTPUT,
  );
  const callSpeaker = findDevice(devices, "audioinput", CALL_SPEAKER_INPUT);
  const missing = [
    [CALL_MICROPHONE_OUTPUT, callMicrophone],
    [CALL_SPEAKER_INPUT, callSpeaker],
  ]
    .filter(([, device]) => !device)
    .map(([name]) => name);
  if (missing.length > 0)
    throw new Error(
      `The meeting needs the virtual audio devices ${CALL_MICROPHONE_OUTPUT} and ${CALL_SPEAKER_INPUT}; missing: ${missing.join(", ")}`,
    );
  return { callMicrophone, callSpeaker };
}

export function chooseSpeaker(devices) {
  const outputs = devicesOfKind(devices, "audiooutput");
  const current = currentDevice(outputs);
  if (isCallDevice(current)) return builtInId(outputs);
  return realDeviceId(outputs, current);
}

function openCallSpeaker(device) {
  return navigator.mediaDevices.getUserMedia({
    audio: {
      deviceId: { exact: device.deviceId },
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
    },
  });
}

async function playCallToUser(callAudio, speaker) {
  const earpiece = new Audio();
  await earpiece.setSinkId(speaker);
  earpiece.srcObject = callAudio;
  await earpiece.play();
  return earpiece;
}

function openMeetingSocket() {
  const socket = new WebSocket(`ws://${location.host}/meeting`);
  socket.binaryType = "arraybuffer";
  return new Promise((resolve, reject) => {
    socket.onopen = () => resolve(socket);
    socket.onerror = () =>
      reject(new Error("The meeting socket did not open on the avatar"));
  });
}

function sendCallFrames(capture, callAudio, socket) {
  const frames = new AudioWorkletNode(capture, "microphone-frames", {
    channelCount: 1,
    channelCountMode: "explicit",
  });
  capture.createMediaStreamSource(callAudio).connect(frames);
  frames.port.onmessage = (event) => {
    if (socket.readyState === WebSocket.OPEN) socket.send(event.data);
  };
}

async function openCallMicrophone(output, microphone, voiceForCall) {
  const context = new AudioContext();
  await context.setSinkId(output.deviceId);
  const gains = {};
  const streams = { human: microphone, agent: voiceForCall.stream };
  for (const [channel, stream] of Object.entries(streams)) {
    gains[channel] = context.createGain();
    context.createMediaStreamSource(stream).connect(gains[channel]);
    gains[channel].connect(context.destination);
  }
  await context.resume();
  return { context, gains };
}

function applyLevels(gains, levels) {
  for (const [channel, level] of Object.entries(levels))
    if (Object.hasOwn(gains, channel)) gains[channel].gain.value = level;
}

export async function joinMeeting({
  microphone,
  playback,
  voice,
  openAudioGraph,
}) {
  const devices = await navigator.mediaDevices.enumerateDevices();
  const { callMicrophone, callSpeaker } = requireCallDevices(devices);
  const speaker = chooseSpeaker(devices);
  if (!speaker)
    throw new Error(
      "The meeting found no speaker of the user's own to play the call on",
    );
  await playback.setSinkId(speaker);
  const callAudio = await openCallSpeaker(callSpeaker);
  const earpiece = await playCallToUser(callAudio, speaker);
  const capture = await openAudioGraph(CALL_RATE);
  const socket = await openMeetingSocket();
  sendCallFrames(capture, callAudio, socket);
  const voiceForCall = playback.createMediaStreamDestination();
  voice.connect(voiceForCall);
  const { context: toCall, gains } = await openCallMicrophone(
    callMicrophone,
    microphone,
    voiceForCall,
  );
  return {
    closed: new Promise((resolve) => {
      socket.onclose = resolve;
    }),
    setLevels: (levels) => applyLevels(gains, levels),
    outputsByDisplayCopy: { stage: toCall },
    leave() {
      for (const track of callAudio.getTracks()) track.stop();
      earpiece.srcObject = null;
      socket.onclose = null;
      socket.close();
      for (const graph of [capture, toCall])
        if (graph.state !== "closed") graph.close();
    },
  };
}
