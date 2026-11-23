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
    hostname = cfg.hostName;
    userProfilePath = ":${config.home.profileDirectory}/bin";
  };
  characters = import ../../runtime/cast/characters.nix { inherit pkgs; };
  characterVoices = import ../../runtime/cast/character-voices.nix { inherit pkgs; };
  transcriptionModel = import ../../runtime/home-manager/avatar-transcription-model.nix {
    inherit pkgs;
  };
  tools = lib.makeBinPath [
    programs.audio
    cfg.speechPackage
    pkgs.ffmpeg
    pkgs.piper-tts
    pkgs.pulseaudio
  ];
  serviceUnit = {
    PartOf = [ "avatar.target" ];
    After = [ "avatar-audio.service" ];
    Requires = [ "avatar-audio.service" ];
  };
in
{
  config = lib.mkIf (cfg.enable && cfg.role == "host") {
    home.packages = with programs; [
      audio
      command
      controls
      floor
      listen
      start
      stop
      talk
      transcribe
    ];
    systemd.user = {
      targets.avatar.Unit = {
        Description = "Local avatar media";
        Requires = [
          "avatar-audio.service"
          "avatar-control-server.service"
          "avatar-renderer.service"
          "avatar-browser.service"
          "avatar-camera.service"
        ];
      };
      targets.avatar-conversation.Unit = {
        Description = "Avatar desktop conversation";
        Requires = [
          "avatar-audio.service"
          "avatar-control-server.service"
        ];
        Wants = [ "avatar-room-microphone.service" ];
      };
      services = {
        avatar-audio = {
          Unit = {
            Description = "Isolated avatar audio mix";
            PartOf = [ "avatar.target" ];
          };
          Service = {
            Type = "oneshot";
            RemainAfterExit = true;
            ExecStart = "${programs.audio}/bin/avatar-audio start";
            ExecStop = "${programs.audio}/bin/avatar-audio stop";
          };
        };
        avatar-audio-recovery = {
          Unit = {
            Description = "Avatar audio mix and room microphone brought back after PipeWire restarts";
            After = [ "pipewire-pulse.service" ];
            PartOf = [ "pipewire-pulse.service" ];
          };
          Service = {
            Type = "oneshot";
            ExecStart = [
              "${programs.audio}/bin/avatar-audio recover"
              "${pkgs.systemd}/bin/systemctl --user --no-block try-restart avatar-room-microphone.service"
            ];
            Restart = "on-failure";
            RestartSec = 2;
          };
          Install.WantedBy = [ "pipewire-pulse.service" ];
        };
        avatar-control-server = {
          Unit = serviceUnit // {
            Description = "Avatar control server";
          };
          Service = {
            ExecStart = "${programs.avatarExecutable} control";
            Environment = [
              "PATH=${tools}:${config.home.profileDirectory}/bin"
              "AVATAR_CLOUD_VOICE=${(builtins.head characters).cloudVoice}"
              "AVATAR_CHARACTER_VOICES=${characterVoices}"
              "AVATAR_TTS_VOICE=pt_BR-faber-medium"
              "AVATAR_TTS_VOICE_DIRECTORY=${import ../../runtime/speech-voices.nix { inherit pkgs; }}"
              "AVATAR_AUDIO_MONITOR=AvatarLipSync.monitor"
              "XDG_RUNTIME_DIR=/run/user/%U"
            ];
            Restart = "on-failure";
          };
          Install.WantedBy = [ "avatar.target" ];
        };
        avatar-room-microphone = {
          Unit = {
            Description = "Avatar microphone with desktop audio cancelled";
            PartOf = [ "avatar-conversation.target" ];
          };
          Service = {
            ExecStart = "${pkgs.pipewire}/bin/pipewire -c ${../../runtime/room-microphone.conf}";
            Restart = "on-failure";
          };
        };
        avatar-transcriber = {
          Unit = {
            Description = "Local avatar speech transcription";
            StopWhenUnneeded = true;
          };
          Service = {
            ExecStart = "${pkgs.whisper-cpp-vulkan}/bin/whisper-server --host 127.0.0.1 --port 8767 --model ${transcriptionModel} --language auto --threads 4";
            Restart = "on-failure";
          };
        };
        avatar-listener = {
          Unit = {
            Description = "Local avatar meeting listener";
            PartOf = [ "avatar-conversation.target" ];
            After = [ "hex.service" ];
            Wants = [ "hex.service" ];
          };
          Service = {
            ExecStart = toString programs.listener;
            Restart = "on-failure";
          };
        };
        avatar-live = {
          Unit = {
            Description = "Avatar live duplex voice";
            PartOf = [ "avatar-conversation.target" ];
            StartLimitIntervalSec = 120;
            StartLimitBurst = 3;
          };
          Service = {
            ExecStart = toString programs.live;
            Environment = [
              "AVATAR_CHARACTER_VOICES=${characterVoices}"
              "AVATAR_SERVANT_CHARACTERS=${programs.servantCharacters}"
            ];
            Restart = "on-failure";
            RestartSec = 5;
          };
        };
      };
    };
  };
}
