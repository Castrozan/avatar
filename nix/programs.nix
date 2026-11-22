{
  pkgs,
  lib,
  avatar,
  hostname ? null,
  userProfilePath ? "",
}:
let
  herdr = avatar.herdrPackage;
  a2a = avatar.a2aPackage;
  a2aLibrary = lib.concatMapStringsSep ":" (directory: "${directory}") avatar.peerIdentityPythonPath;
  servantCharacters = ../runtime/cast/servant-characters.txt;
  ordinaryWords = import ../runtime/scripts/listening/transcription/ordinary-words.nix {
    inherit pkgs;
  };
  sendCommand = pkgs.writeShellScript "avatar-send-command" ''
    export NODE_PATH=${control}/node_modules
    exec ${pkgs.nodejs_22}/bin/node ${../runtime/scripts/command.js} "$@"
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
      a2aLibrary
      ;
  };
  transcribe = pkgs.writeShellScriptBin "avatar-transcribe" ''
    export PYTHONPATH=${../runtime/scripts/listening}
    exec ${pkgs.python312}/bin/python3 -m transcription.transcriber
  '';
  control = import ./packages/control-server.nix { inherit pkgs; };
  renderer = import ./packages/renderer.nix { inherit pkgs; };
  speakerModel = pkgs.fetchurl {
    url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx";
    sha256 = "18pb1kqg82x3qa07yd4b6n6yjmvbdmfh7xckjdm5h41sjqbgqg5a";
  };
  turnModel = import ../runtime/home-manager/avatar-turn-model.nix { inherit pkgs; };
  websocketPython = pkgs.python312.withPackages (packages: [ packages.websockets ]);
  listenerPython = pkgs.python3.withPackages (
    packages: with packages; [
      numpy
      onnxruntime
      sherpa-onnx
    ]
  );
  audio = pkgs.writeShellScriptBin "avatar-audio" ''
    export PATH=${lib.makeBinPath [ pkgs.pulseaudio ]}:$PATH
    export PYTHONPATH=${../runtime/scripts/audio_graph}
    exec ${pkgs.python312}/bin/python3 ${../runtime/scripts/audio_graph/audio_cli.py} "$@"
  '';
  serve = pkgs.writeShellScript "avatar-renderer" ''
    export AVATAR_RENDERER_ROOT=${renderer}
    exec ${pkgs.python312}/bin/python3 ${../runtime/scripts/serve.py}
  '';
  browserRuntime = ../skills/avatar/scripts/browser/runtime.py;
  camera = pkgs.writeShellScript "avatar-camera" ''
    export CDP_PORT="$(${pkgs.python312}/bin/python3 ${browserRuntime} endpoint)"
    export PATH=${
      lib.makeBinPath [
        pkgs.ffmpeg
        pkgs.chromium
      ]
    }:$PATH
    exec ${pkgs.nodejs_22}/bin/node ${control}/virtual-camera.js --fps 60 --width 1280 --height 720
  '';
  controls = pkgs.writeShellScriptBin "avatar-controls" ''
    export PATH=${lib.makeBinPath [ pkgs.chromium ]}:$PATH
    exec ${pkgs.python312}/bin/python3 ${browserRuntime} controls
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
    export PYTHONPATH=${../runtime/scripts}:${../runtime/scripts/audio_graph}:${../runtime/scripts/listening}:${../runtime/scripts/echo}:${../runtime/scripts/conversation}
    exec ${pkgs.python312}/bin/python3 ${../runtime/scripts/meetings/start.py} "$@"
  '';
  stop = pkgs.writeShellScriptBin "avatar-stop" ''
    export PATH=${
      lib.makeBinPath [
        audio
        pkgs.systemd
      ]
    }:$PATH
    ${builtins.readFile ../runtime/scripts/avatar-stop}
  '';
  listen = pkgs.writeShellScriptBin "avatar-listen" ''
    export PATH=${
      lib.makeBinPath [
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH
    export PYTHONPATH=${../runtime/scripts/listening}:${../runtime/scripts/echo}:${../runtime/scripts/conversation}
    exec ${pkgs.python312}/bin/python3 ${../runtime/scripts/listening/listen_cli.py} "$@"
  '';
  listener = pkgs.writeShellScript "avatar-listener" ''
    export PATH=${
      lib.makeBinPath [
        command
        pkgs.pulseaudio
      ]
    }:$PATH${userProfilePath}
    export AVATAR_SPEAKER_MODEL=${speakerModel}
    export AVATAR_TURN_MODEL=${turnModel}
    export AVATAR_ORDINARY_WORDS=${ordinaryWords}
    export AVATAR_SERVANT_ROSTER=${avatar.servantRoster}
    export PYTHONPATH=${../runtime/scripts/listening}:${../runtime/scripts/echo}
    exec ${listenerPython}/bin/python3 ${../runtime/scripts/listening/listener.py}
  '';
  live = pkgs.writeShellScript "avatar-live" ''
    export PATH=${
      lib.makeBinPath [
        a2a
        pkgs.openssh
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH
    export PYTHONPATH=${../runtime/scripts/live}:${../runtime/scripts/listening}:${../runtime/scripts/echo}:${../runtime/scripts/conversation}:${../runtime/scripts/wake}
    exec ${websocketPython}/bin/python3 ${../runtime/scripts/live/live_bridge.py}
  '';
  remoteSeat = pkgs.writeShellScript "avatar-remote-seat" ''
    export PATH=${
      lib.makeBinPath [
        pkgs.pulseaudio
        pkgs.systemd
      ]
    }:$PATH
    export AVATAR_SEAT_PAGE=${../runtime/scripts/conversation/remote_seat/page}
    export PYTHONPATH=${../runtime/scripts/audio_graph}:${../runtime/scripts/listening}
    exec ${websocketPython}/bin/python3 ${../runtime/scripts/conversation/remote_seat}/seat_bridge.py
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
    export PYTHONPATH=${../runtime/scripts}:${../runtime/scripts/audio_graph}:${../runtime/scripts/listening}:${../runtime/scripts/echo}:${../runtime/scripts/conversation}:${../runtime/scripts/wake}
    exec ${pkgs.python312}/bin/python3 ${../runtime/scripts/conversation/talk_cli.py} "$@"
  '';
in
{
  inherit
    a2a
    a2aLibrary
    audio
    browserRuntime
    camera
    command
    control
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
    ;
}
