{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.avatar;
  programs = import ../programs.nix {
    inherit pkgs lib;
    avatar = cfg;
  };
in
{
  config = lib.mkIf (cfg.enable && cfg.role == "host") {
    systemd.user = {
      targets.avatar-window.Unit = {
        Description = "Avatar window shown in a desktop conversation";
        PartOf = [ "avatar-conversation.target" ];
        BindsTo = [ "avatar-browser.service" ];
        Requires = [ "avatar-window-reveal.service" ];
        After = [
          "avatar-browser.service"
          "avatar-window-reveal.service"
        ];
      };
      services = {
        avatar-window-reveal = {
          Unit = {
            Description = "Avatar window revealed every time its browser starts";
            PartOf = [ "avatar-window.target" ];
            After = [ "avatar-browser.service" ];
          };
          Service = {
            Type = "oneshot";
            ExecStart = toString programs.windowReveal;
          };
        };
        avatar-renderer = {
          Unit = {
            Description = "Offline avatar renderer";
            PartOf = [ "avatar.target" ];
            After = [ "avatar-audio.service" ];
            Requires = [ "avatar-audio.service" ];
            StopWhenUnneeded = true;
          };
          Service = {
            ExecStart = toString programs.serve;
            Restart = "on-failure";
          };
          Install.WantedBy = [ "avatar.target" ];
        };
        avatar-browser = {
          Unit = {
            Description = "Avatar output browser";
            PartOf = [ "avatar.target" ];
            After = [
              "avatar-renderer.service"
              "avatar-control-server.service"
            ];
            Requires = [
              "avatar-renderer.service"
              "avatar-control-server.service"
            ];
            StopWhenUnneeded = true;
          };
          Service = {
            ExecStart = "${pkgs.python312}/bin/python3 ${programs.browserRuntime} supervise http://localhost:3000/?output=1";
            ExecStopPost = "${pkgs.python312}/bin/python3 ${programs.browserRuntime} stop";
            Environment = [ "PATH=${lib.makeBinPath [ pkgs.chromium ]}" ];
            KillMode = "mixed";
          };
          Install.WantedBy = [ "avatar.target" ];
        };
        avatar-camera = {
          Unit = {
            Description = "Avatar virtual camera";
            PartOf = [ "avatar.target" ];
            After = [ "avatar-browser.service" ];
            Requires = [ "avatar-browser.service" ];
          };
          Service = {
            ExecStart = toString programs.camera;
            Environment = [ "V4L2_DEVICE=/dev/video10" ];
            Restart = "always";
          };
          Install.WantedBy = [ "avatar.target" ];
        };
      };
    };
  };
}
