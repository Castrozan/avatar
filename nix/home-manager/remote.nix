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
  inherit (cfg.mac) browserApplication;
  conversation = import ./mac-conversation.nix {
    inherit pkgs lib programs;
    inherit browserApplication;
    inherit (cfg) servantRoster;
    host = cfg.topology.host;
    hostname = cfg.hostName;
    speech = cfg.speechPackage;
  };
  inherit (conversation) talk;
  peerHosts = lib.filter (machine: machine != cfg.hostName) cfg.topology.machines;
  wake = pkgs.writeShellScriptBin "avatar-wake" ''
    export PATH=${
      lib.makeBinPath [
        programs.a2a
        programs.herdr
        conversation.command
        talk
      ]
    }:$PATH
    export AVATAR_HOST=${cfg.topology.host}
    export AVATAR_AGENT_HOST=${cfg.hostName}
    export AVATAR_PEER_HOSTS="${lib.concatStringsSep " " peerHosts}"
    export AVATAR_ORDINARY_WORDS=${programs.ordinaryWords}
    export AVATAR_SERVANT_CHARACTERS=${programs.servantCharacters}
    export AVATAR_WAKE_PAGE=${../../runtime/scripts/wake/page}
    export AVATAR_BROWSER_APPLICATION="${browserApplication}"
    export AVATAR_SEAT_PAGE=${../../runtime/scripts/conversation/remote_seat/page}
    exec ${programs.avatarExecutable} mac wake "$@"
  '';
  doctor = import ../../runtime/home-manager/avatar-doctor-package.nix {
    inherit pkgs lib;
    commands = [
      talk
      programs.transcribe
    ];
    inherit (programs) avatarExecutable;
    arguments = "--seat-of ${cfg.topology.host}";
  };
  switchLatency = import ../../runtime/home-manager/avatar-switch-latency-package.nix {
    inherit pkgs;
    inherit (programs) avatarExecutable;
  };
  conversationLog = "${config.home.homeDirectory}/Library/Logs/avatar-conversation.log";
  onDemandAgent =
    name: package: executableName:
    cfg.mac.launchAgent {
      inherit name package executableName;
      label = "com.dotfiles.${name}";
      serviceConfig = {
        KeepAlive = false;
        RunAtLoad = false;
        StandardOutPath = conversationLog;
        StandardErrorPath = conversationLog;
      };
    };
  wakeLog = "${config.home.homeDirectory}/Library/Logs/avatar-wake.log";
  wakeLaunchAgent = cfg.mac.launchAgent {
    name = "avatar-wake";
    label = "com.dotfiles.avatar-wake";
    package = wake;
    executableName = "avatar-wake";
    restartWhenPackageChanges = true;
    serviceConfig = {
      KeepAlive = true;
      RunAtLoad = true;
      StandardOutPath = wakeLog;
      StandardErrorPath = wakeLog;
    };
  };
  launchAgents = [
    wakeLaunchAgent
    (onDemandAgent "avatar-conversation" conversation.listener "avatar-conversation-listener")
    (onDemandAgent "avatar-live" conversation.live "avatar-live")
    (onDemandAgent "avatar-control" conversation.controlServer "avatar-control-server")
    (onDemandAgent "avatar-renderer" conversation.renderer "avatar-renderer-server")
  ];
  homeAndLaunchdOnly = agent: {
    home = agent.home or { };
    launchd = agent.launchd or { };
  };
in
{
  config = lib.mkIf (cfg.enable && cfg.role == "remote") (
    lib.mkMerge (
      [
        {
          home.packages = [
            talk
            wake
            doctor
            switchLatency
            programs.transcribe
            conversation.command
            (conversation.meetingCommand "avatar-start" "meeting start")
            (conversation.meetingCommand "avatar-stop" "meeting stop")
            (conversation.meetingCommand "avatar-controls" "meeting controls")
            (conversation.meetingCommand "avatar-listen" "listen")
          ];
          launchd.agents.avatar-switch-latency = {
            enable = true;
            config = {
              Label = "com.dotfiles.avatar-switch-latency";
              ProgramArguments = [
                "${switchLatency}/bin/avatar-switch-latency"
                "--record"
              ];
              RunAtLoad = false;
              StartCalendarInterval = [
                {
                  Hour = 0;
                  Minute = 0;
                }
              ];
              StandardOutPath = wakeLog;
              StandardErrorPath = wakeLog;
            };
          };
        }
      ]
      ++ map homeAndLaunchdOnly launchAgents
    )
  );
}
