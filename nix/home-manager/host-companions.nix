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
  programsSpeakingForThisHost = import ../programs.nix {
    inherit pkgs lib;
    avatar = cfg;
    hostname = cfg.hostName;
  };
  keepRunningThroughRebuild =
    (import ../../runtime/home-manager/avatar-rebuild-home-manager.nix {
      inherit lib;
      isNixOS = true;
    }).config;
  doctor = import ../../runtime/home-manager/avatar-doctor-package.nix {
    inherit pkgs lib;
    commands = [
      programs.transcribe
      pkgs.systemd
    ];
    inherit (programs) avatarExecutable;
  };
  voiceHolder = pkgs.writeShellScriptBin "avatar-voice-holder" ''
    export PATH=${
      lib.makeBinPath [
        programsSpeakingForThisHost.talk
        programsSpeakingForThisHost.command
        pkgs.systemd
      ]
    }:$PATH:${config.home.profileDirectory}/bin
    export AVATAR_SERVANT_CHARACTERS=${programsSpeakingForThisHost.servantCharacters}
    exec ${programsSpeakingForThisHost.avatarExecutable} holder "$@"
  '';
in
{
  config = lib.mkIf (cfg.enable && cfg.role == "host") (
    lib.mkMerge [
      keepRunningThroughRebuild
      {
        home.packages = [
          doctor
          voiceHolder
        ];
        systemd.user.services.avatar-remote-seat = {
          Unit.Description = "Avatar microphone and voice lent by another machine";
          Service = {
            ExecStart = toString programs.remoteSeat;
            ExecStopPost = "${pkgs.systemd}/bin/systemctl --user --no-block stop avatar-conversation.target avatar-listener.service";
          };
        };
      }
    ]
  );
}
