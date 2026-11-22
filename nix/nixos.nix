{ config, lib, ... }:
{
  options.services.avatar.virtualCamera.enable =
    lib.mkEnableOption "the v4l2loopback device the avatar streams to as Avatar Cam on /dev/video10";

  config = lib.mkIf config.services.avatar.virtualCamera.enable {
    boot = {
      extraModulePackages = [ config.boot.kernelPackages.v4l2loopback ];
      kernelModules = [ "v4l2loopback" ];
      extraModprobeConfig = ''
        options v4l2loopback devices=1 video_nr=10 card_label="Avatar Cam" exclusive_caps=1
      '';
    };
  };
}
