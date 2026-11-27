{
  pkgs,
  lib,
  avatar,
  hostname ? null,
  userProfilePath ? "",
}:
let
  avatarExecutable = "${avatar.rustPackage}/bin/avatar";
  herdr = avatar.herdrPackage;
  a2a = avatar.a2aPackage;
  servantCharacters = ../runtime/cast/servant-characters.txt;
  ordinaryWords = import ../runtime/scripts/listening/transcription/ordinary-words.nix {
    inherit pkgs;
  };
  sendCommand = pkgs.writeShellScript "avatar-send-command" ''
    exec ${avatarExecutable} command "$@"
  '';
  unclaimedCommand = pkgs.writeShellScriptBin "avatar-command" ''
    exec ${sendCommand} "$@"
  '';
  floor = import ../runtime/home-manager/avatar-floor-package.nix {
    inherit
      pkgs
      lib
      herdr
      unclaimedCommand
      servantCharacters
      a2a
      avatarExecutable
      ;
  };
  transcribe = pkgs.writeShellScriptBin "avatar-transcribe" ''
    exec ${avatarExecutable} transcribe
  '';
  renderer = import ./packages/renderer.nix { inherit pkgs; };
  speakerModel = (import ../runtime/assets { inherit pkgs; }).fetch "speaker-embedding";
  turnModel = import ../runtime/home-manager/avatar-turn-model.nix { inherit pkgs; };
  voiceActivityModel = import ../runtime/home-manager/avatar-voice-activity-model.nix {
    inherit pkgs;
  };
  audio = pkgs.writeShellScriptBin "avatar-audio" ''
    export PATH=${lib.makeBinPath [ pkgs.pulseaudio ]}:$PATH
    exec ${avatarExecutable} audio "$@"
  '';
  serve = pkgs.writeShellScript "avatar-renderer" ''
    export AVATAR_RENDERER_ROOT=${renderer}
    exec ${avatarExecutable} serve
  '';
  camera = pkgs.writeShellScript "avatar-camera" ''
    export CDP_PORT="$(${avatarExecutable} browser endpoint)"
    export PATH=${
      lib.makeBinPath [
        pkgs.ffmpeg
        pkgs.chromium
      ]
    }:$PATH
    exec ${avatarExecutable} camera --fps 60 --width 1280 --height 720
  '';
  controls = pkgs.writeShellScriptBin "avatar-controls" ''
    export PATH=${lib.makeBinPath [ pkgs.chromium ]}:$PATH
    exec ${avatarExecutable} browser controls
  '';
  command = pkgs.writeShellScriptBin "avatar-command" ''
    if [ "''${1:-}" = speak ]; then
      ${floor}/bin/avatar-claim-floor || exit 1
    fi
    exec ${sendCommand} "$@"
  '';
  start = pkgs.writeShellScriptBin "avatar-start" ''
    export PATH=${
      lib.makeBinPath [
        audio
        pkgs.systemd
        pkgs.v4l-utils
      ]
    }:$PATH
    exec ${avatarExecutable} start "$@"
  '';
  stop = pkgs.writeShellScriptBin "avatar-stop" ''
    export PATH=${
      lib.makeBinPath [
        audio
        pkgs.systemd
      ]
    }:$PATH
    exec ${avatarExecutable} stop
  '';
  listen = pkgs.writeShellScriptBin "avatar-listen" ''
    export PATH=${
      lib.makeBinPath [
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH
    exec ${avatarExecutable} listen "$@"
  '';
  listener = pkgs.writeShellScript "avatar-listener" ''
    export PATH=${lib.makeBinPath [ pkgs.pulseaudio ]}:$PATH${userProfilePath}
    export AVATAR_SPEAKER_MODEL=${speakerModel}
    export AVATAR_TURN_MODEL=${turnModel}
    export AVATAR_VOICE_ACTIVITY_MODEL=${voiceActivityModel}
    export AVATAR_ORDINARY_WORDS=${ordinaryWords}
    export AVATAR_SERVANT_ROSTER=${avatar.servantRoster}
    exec ${avatarExecutable} listener
  '';
  live = pkgs.writeShellScript "avatar-live" ''
    export PATH=${
      lib.makeBinPath [
        a2a
        pkgs.openssh
        pkgs.pulseaudio
      ]
    }:$PATH
    exec ${avatarExecutable} live
  '';
  remoteSeat = pkgs.writeShellScript "avatar-remote-seat" ''
    export PATH=${
      lib.makeBinPath [
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH
    export AVATAR_SEAT_PAGE=${../runtime/scripts/conversation/remote_seat/page}
    exec ${avatarExecutable} remote-seat
  '';
  talk = pkgs.writeShellScriptBin "avatar-talk" ''
    export PATH=${
      lib.makeBinPath [
        audio
        command
        floor
        stop
        pkgs.hyprland
        pkgs.openssh
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH${userProfilePath}
    ${lib.optionalString (hostname != null) "export AVATAR_AGENT_HOST=${hostname}"}
    export AVATAR_SERVANT_CHARACTERS=${servantCharacters}
    exec ${avatarExecutable} talk "$@"
  '';
  windowReveal = pkgs.writeShellScript "avatar-window-reveal" ''
    export PATH=${
      lib.makeBinPath [
        command
        pkgs.hyprland
        pkgs.systemd
      ]
    }:$PATH
    exec ${avatarExecutable} window-reveal
  '';
in
{
  inherit
    a2a
    audio
    avatarExecutable
    camera
    command
    controls
    floor
    herdr
    listen
    listener
    live
    ordinaryWords
    remoteSeat
    serve
    servantCharacters
    start
    stop
    talk
    transcribe
    unclaimedCommand
    windowReveal
    ;
}
