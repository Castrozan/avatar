{
  pkgs,
  lib,
  programs,
  host,
  hostname,
  browserApplication,
  servantRoster,
  speech,
}:
let
  scripts = ../../runtime/scripts;
  characters = import ../../runtime/cast/characters.nix { inherit pkgs; };
  environment = ''
    export AVATAR_HOST=${host}
    export AVATAR_AGENT_HOST=${hostname}
    export AVATAR_SEAT_PAGE=${scripts}/conversation/remote_seat/page
    export AVATAR_WAKE_PAGE=${scripts}/wake/page
    export AVATAR_BROWSER_APPLICATION="${browserApplication}"
    export AVATAR_SERVANT_CHARACTERS=${programs.servantCharacters}
    export AVATAR_TURN_MODEL=${
      import ../../runtime/home-manager/avatar-turn-model.nix { inherit pkgs; }
    }
    export AVATAR_VOICE_ACTIVITY_MODEL=${
      import ../../runtime/home-manager/avatar-voice-activity-model.nix { inherit pkgs; }
    }
    export AVATAR_ORDINARY_WORDS=${programs.ordinaryWords}
    export AVATAR_SERVANT_ROSTER=${servantRoster}
  '';
  seatPlay = pkgs.writeShellScriptBin "avatar-seat-play" ''
    ${environment}
    exec ${programs.avatarExecutable} mac play "$@"
  '';
in
{
  command = programs.unclaimedCommand;
  controlServer = pkgs.writeShellScriptBin "avatar-control-server" ''
    export PATH=${
      lib.makeBinPath [
        speech
        seatPlay
      ]
    }:$PATH
    export AVATAR_CLOUD_VOICE=${(builtins.head characters).cloudVoice}
    export AVATAR_CHARACTER_VOICES=${import ../../runtime/cast/character-voices.nix { inherit pkgs; }}
    exec ${programs.avatarExecutable} control
  '';
  renderer = pkgs.writeShellScriptBin "avatar-renderer-server" ''
    exec ${programs.serve}
  '';
  talk = pkgs.writeShellScriptBin "avatar-talk" ''
    export PATH=${
      lib.makeBinPath [
        programs.unclaimedCommand
        programs.floor
      ]
    }:$PATH
    ${environment}
    exec ${programs.avatarExecutable} mac talk "$@"
  '';
  meetingCommand =
    name: subcommand:
    pkgs.writeShellScriptBin name ''
      export PATH=${
        lib.makeBinPath [
          programs.unclaimedCommand
          pkgs.switchaudio-osx
        ]
      }:$PATH
      ${environment}
      exec ${programs.avatarExecutable} mac ${subcommand} "$@"
    '';
  listener = pkgs.writeShellScriptBin "avatar-conversation-listener" ''
    ${environment}
    exec ${programs.avatarExecutable} listener
  '';
  live = pkgs.writeShellScriptBin "avatar-live" ''
    export PATH=${
      lib.makeBinPath [
        programs.a2a
        programs.herdr
      ]
    }:$PATH
    ${environment}
    export AVATAR_CHARACTER_VOICES=${import ../../runtime/cast/character-voices.nix { inherit pkgs; }}
    exec ${programs.avatarExecutable} live
  '';
}
