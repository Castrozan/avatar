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
    home.packages = [ wake ];
    systemd.user.services.avatar-wake = {
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
}
