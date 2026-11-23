{
  config,
  lib,
  pkgs,
  ...
}:
{
  options.services.avatar.callDevices.enable =
    lib.mkEnableOption "the BlackHole call microphone and speaker drivers and OBS for the avatar's meetings";

  options.services.avatar.callDevices.package = lib.mkOption {
    type = lib.types.package;
    description = "Package providing `bin/avatar`, whose `mac install-call-audio-drivers` installs the call drivers; the flake sets it to its own build.";
  };

  config = lib.mkIf config.services.avatar.callDevices.enable (
    import ../runtime/avatar-call-devices-nix-darwin.nix {
      inherit lib pkgs;
      avatar = config.services.avatar.callDevices.package;
    }
  );
}
