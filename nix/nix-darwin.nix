{
  config,
  lib,
  pkgs,
  ...
}:
{
  options.services.avatar.callDevices.enable =
    lib.mkEnableOption "the BlackHole call microphone and speaker drivers and OBS for the avatar's meetings";

  config = lib.mkIf config.services.avatar.callDevices.enable (
    import ../runtime/avatar-call-devices-nix-darwin.nix { inherit lib pkgs; }
  );
}
