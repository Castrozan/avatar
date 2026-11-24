const VIRTUAL_DEFAULTS = new Set(["default", "communications"]);
const CALL_DEVICE_NAME = "BlackHole";

function labelled(device, word) {
  return device.label.toLowerCase().includes(word);
}

function isConcrete(device) {
  return !VIRTUAL_DEFAULTS.has(device.deviceId);
}

function isBuiltIn(device) {
  return isConcrete(device) && labelled(device, "built-in");
}

function isBluetooth(device) {
  return device !== undefined && labelled(device, "bluetooth");
}

export function isCallDevice(device) {
  return device !== undefined && device.label.includes(CALL_DEVICE_NAME);
}

export function devicesOfKind(devices, kind) {
  return devices.filter((device) => device.kind === kind);
}

export function currentDevice(devices) {
  return devices.find((device) => device.deviceId === "default") ?? devices[0];
}

export function builtInId(devices) {
  return devices.find(isBuiltIn)?.deviceId ?? null;
}

function standsFor(entry, candidate) {
  if (!isConcrete(candidate)) return false;
  if (candidate.groupId === entry.groupId) return true;
  return candidate.label !== "" && entry.label.endsWith(candidate.label);
}

export function realDeviceId(devices, device) {
  if (device === undefined) return null;
  if (isConcrete(device)) return device.deviceId;
  return (
    devices.find((candidate) => standsFor(device, candidate))?.deviceId ?? null
  );
}

export function chooseMicrophone(
  devices,
  { pinned = false, previous = null } = {},
) {
  const inputs = devicesOfKind(devices, "audioinput");
  const current = currentDevice(inputs);
  if (isBluetooth(current)) return builtInId(inputs);
  const present = inputs.some((device) => device.deviceId === previous);
  if (isCallDevice(current)) return present ? previous : builtInId(inputs);
  return pinned ? realDeviceId(inputs, current) : null;
}
