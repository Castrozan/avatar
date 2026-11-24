import { chooseMicrophone } from "/microphone-choice.js";

function requestMicrophone(mediaDevices, deviceId) {
  return mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
      ...(deviceId && { deviceId: { exact: deviceId } }),
    },
  });
}

export async function chosenMicrophone(mediaDevices, choice) {
  return chooseMicrophone(await mediaDevices.enumerateDevices(), choice);
}

export function openMicrophoneOn(mediaDevices, deviceId) {
  return requestMicrophone(mediaDevices, deviceId);
}

export async function openMicrophone(mediaDevices, choice) {
  return requestMicrophone(
    mediaDevices,
    await chosenMicrophone(mediaDevices, choice),
  );
}
