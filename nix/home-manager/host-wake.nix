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
  switchLatency = import ../../runtime/home-manager/avatar-switch-latency-package.nix {
    inherit pkgs;
  };
  peerHosts = lib.filter (machine: machine != cfg.hostName) cfg.topology.machines;
  wake = pkgs.writeShellScriptBin "avatar-wake" ''
    export PATH=${
      lib.makeBinPath [
        programs.a2a
        programs.command
        programs.herdr
        programs.talk
        pkgs.openssh
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH:${config.home.profileDirectory}/bin
    export AVATAR_AGENT_HOST=${cfg.hostName}
    export AVATAR_PEER_HOSTS="${lib.concatStringsSep " " peerHosts}"
    export AVATAR_SERVANT_CHARACTERS=${programs.servantCharacters}
    export AVATAR_ORDINARY_WORDS=${programs.ordinaryWords}
    export PYTHONPATH=${../../runtime/scripts/listening}:${../../runtime/scripts/echo}:${../../runtime/scripts/conversation}:${../../runtime/scripts/wake}:${../../runtime/scripts/addressing}:${programs.a2aLibrary}
    exec ${pkgs.python312}/bin/python3 ${../../runtime/scripts/wake/wake_listener.py} "$@"
  '';
in
{
  config = lib.mkIf (cfg.enable && cfg.role == "host") {
    home.packages = [
      wake
      switchLatency
    ];
    systemd.user = {
      services = {
        avatar-switch-latency = {
          Unit.Description = "Keep the measured avatar switch round trips past wake log rotation";
          Service = {
            Type = "oneshot";
            ExecStart = "${switchLatency}/bin/avatar-switch-latency --record";
          };
        };
        avatar-wake = {
          Unit = {
            Description = "Wake a live agent session by its Servant name";
            After = [
              "graphical-session.target"
              "hex.service"
            ];
            PartOf = [ "graphical-session.target" ];
            Wants = [ "hex.service" ];
          };
          Service = {
            ExecStart = "${wake}/bin/avatar-wake";
            Restart = "always";
            RestartSec = 5;
          };
          Install.WantedBy = [ "graphical-session.target" ];
        };
      };
      timers.avatar-switch-latency = {
        Unit.Description = "Daily keep of the measured avatar switch round trips";
        Timer = {
          OnCalendar = "daily";
          Persistent = true;
        };
        Install.WantedBy = [ "timers.target" ];
      };
    };
  };
}
