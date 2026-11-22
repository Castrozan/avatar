# Avatar coupling inventory

Measured against dotfiles `main` at `a7de46497b2bffb226320119158358c598e150fc` (2026-11-22); line numbers are from that
commit. In sections 2 to 4, a path that starts with a dotfiles top-level directory (`agent-harness/`,
`machine-configuration/`, `repository/`, `.qlty/`, `.github/`) is relative to the dotfiles root. Any other path, such as
`avatar-programs.nix:9` or `scripts/wake/servant_panes.py:1`, is relative to `machine-configuration/media/avatar/`, which
this repository holds as `runtime/`. `skills/avatar/` is the extracted agent skill.

## 1. Paths that belong to the avatar system

Two directories, 372 tracked files, 39,216 text lines, 7 binary portraits (about 9.1 MB). Nothing else in dotfiles is
avatar code: every other match for "avatar" is a consumer, a name, or unrelated (Chrome profile `avatar_icon`, a YouTube
userscript field).

### `machine-configuration/media/avatar/` (285 files, 32,297 lines) -> `runtime/`

| Area | Files | Lines | Content |
| --- | ---: | ---: | --- |
| top-level files | 10 | 996 | `avatar-home-manager.nix`, `avatar-programs.nix`, `avatar-remote-home-manager.nix`, `avatar-wake-home-manager.nix`, `avatar-call-devices-nix-darwin.nix`, `control-package.nix`, `renderer-package.nix`, `speech-voices.nix`, `room-microphone.conf`, `README.md` |
| `home-manager/` | 11 | 269 | doctor, floor, mac-conversation, rebuild (keep-old), remote-seat, topology, transcription/turn models, voice-holder, control-server tools |
| `cast/` | 4 | 338 | `characters.nix`, `character-voices.nix`, `models.nix`, `servant-characters.txt` |
| `renderer/` | 14 | 1,384 | three-vrm renderer pages and modules |
| `scripts/` (top) | 3 | 169 | `serve.py`, `command.js`, `avatar-stop` |
| `scripts/audio_graph/` | 6 | 553 | PulseAudio graph |
| `scripts/conversation/` | 41 | 3,378 | talk CLI, seats, mac conversation, remote seat page, Discord seat |
| `scripts/doctor/` | 1 | 151 | `avatar-doctor` |
| `scripts/echo/` | 2 | 67 | transcript echo |
| `scripts/holder/` | 3 | 253 | voice holder, Hermes turn |
| `scripts/listening/` | 22 | 1,758 | listener, segmenter, transcription package (Hex, ElevenLabs Scribe) |
| `scripts/live/` | 12 | 979 | GPT live voice bridge |
| `scripts/meetings/` | 2 | 218 | `avatar-start` |
| `scripts/wake/` | 17 | 1,627 | wake listener, floor, Servant panes, mac wake |
| `__tests__/` | 137 | 20,157 | pytest suite (123 `test_*.py`), runners, fixtures, one `.bats` |

### `agent-harness/agent-instructions/skills/workstation/avatar/` (87 files, 6,919 lines) -> `skills/avatar/`

| Area | Files | Lines | Content |
| --- | ---: | ---: | --- |
| `SKILL.md` | 1 | 153 | agent instructions for the `avatar` skill |
| `avatars/` | 7 | binary | portraits linked into the renderer (`renderer-package.nix:24`) |
| `scripts/control-server/` | 50 | 3,532 | Node control server (`buildNpmPackage` in `control-package.nix:5`), incl. `package-lock.json` |
| `scripts/browser/` | 3 | 374 | Chromium runtime used by `avatar-programs.nix:65` and its test |
| `scripts/*.sh` | 6 | 673 | legacy speak/start/stop/capture/readiness helpers |
| `__tests__/` | 20 | 2,187 | 14 unit tests, 3 integration tests, fixtures |

### History carried along

The extraction keeps 160 further historical paths: every earlier name of a current file, found by following `git log
-M` renames backwards. They live under `agents/skills/avatar/`, `agents/avatar/`, `agents/skills/comms/`,
`agents/scripts/`, `static/avatars/` and `agent-harness/agent-instructions/skills/avatar/`, and appear only in commits
before the moves. History reaches back to 2026-02-02 (`feat: avatar controlling skills`).

## 2. Inbound references (rest of dotfiles -> avatar)

| # | Where | What | Kind |
| --- | --- | --- | --- |
| I1 | `machine-configuration/machines/chise/home.nix:69` | imports `media/avatar/avatar-home-manager.nix` | nix import (host) |
| I2 | `machine-configuration/machines/shared-darwin-home-manager.nix:10` | imports `media/avatar/avatar-remote-home-manager.nix` | nix import (kira, rin) |
| I3 | `machine-configuration/machines/shared-darwin-system-nix-darwin.nix:12` | imports `media/avatar/avatar-call-devices-nix-darwin.nix` | nix import (darwin system) |
| I4 | `machine-configuration/machines/chise/system/configs/configuration.nix:31-35` | `v4l2loopback` with `video_nr=10 card_label="Avatar Cam"`; the avatar consumes `/dev/video10` (`avatar-home-manager.nix:130`) | NixOS system setting owned by avatar |
| I5 | `machine-configuration/desktop/desktop-environments/hyprland/program-configuration/conf.d/windows.conf:31` | hides class `avatar-browser` on `special:avatar`; asserted by `skills/avatar/scripts/browser/__tests__/unit/test_runtime.py:8-11` | window-class contract |
| I6 | `agent-harness/harnesses/clawde/discord/voice-bridge/silver-voice-holder.nix:21` | runs `${profileDirectory}/bin/avatar-voice-holder --hermes-profile ... --character darkness-shibu` | executable on profile PATH + character name |
| I7 | `agent-harness/harnesses/clawde/discord/voice-bridge/scripts/silver_channel.py:26`, `voice_setup.py:8`, `silver_voice_cli.py:22` | calls `avatar-talk status`, checks `avatar.target` | CLI + systemd unit name |
| I8 | `agent-harness/agent-instructions/interactive-skill-catalog/skill-set-builders.nix:8-15,29-33` | discovers the skill by `readDir ../skills/workstation` | skill deployment (implicit) |
| I9 | `agent-harness/quality/evaluations/baseline.json:6497` | content hash of `.../workstation/avatar/SKILL.md` | eval evidence |
| I10 | `agent-harness/quality/evaluations/suites/skill_routing.yaml:9` | `avatar` in the generated all-skills routing description | eval suite |
| I11 | `.qlty/ruff.toml:17` | per-file ignore `S310` for `scripts/listening/transcription/cloud_transcriber.py` | lint baseline |
| I12 | `repository/verification/runner/discovery.sh:6-33`, `.github/workflows/python-tests.yml:21-38` | generic `*/__tests__/unit/*` discovery puts avatar tests in the `unit` tier; no avatar-specific entry | CI test tier |
| I13 | `machine-configuration/development/testing/python-test-environment.nix:12-21` | supplies `hypothesis`, `numpy`, `websockets`, `pytest` the avatar tests import | CI test environment |
| I14 | `machine-configuration/audio/__tests__/unit/test_audio_output_switch.py:35,105`, `move-all-streams-to-default-sink.bats:61`, `audio-pipeline.md:54` | `AvatarMic` sink name as fixture data and documentation | name only |
| I15 | `private-configuration/` (private submodule) | recorded user preferences that name the avatar | text only |
| I16 | `agent-harness/orchestration/__tests__/unit/human_facing/*.py`, `skills/agent-workflows/orchestrate/__tests__/evals/decisions.yaml:165,175`, `read-it-later/skill/SKILL.md:39`, `harnesses/claude-code/docs/agent-behavior-audit.md:172`, `development/testing/nix-build-limits-kira-2026-09-16.json:176,179` | the word "avatar" in fixtures, docs and recorded build data | incidental |

Not found: no hooks, no permission rules, no line-count or directory-entry baseline entries, no avatar reference in
`~/zanoni-system`.

## 3. Outbound dependencies (avatar -> rest of dotfiles)

Classification: **moves** with avatar; **input/option** becomes a flake input or module option set by dotfiles;
**interface** stays in dotfiles behind a named contract the avatar consumes.

| # | Dependency | Evidence | Class | Contract after extraction |
| --- | --- | --- | --- | --- |
| O1 | `hostname` specialArg | `avatar-home-manager.nix:7,13`; `avatar-programs.nix:5,166`; `avatar-wake-home-manager.nix:7,13,26`; `avatar-remote-home-manager.nix:6,25,31,43`; `home-manager/avatar-voice-holder-home-manager.nix:7,16`; `home-manager/avatar-mac-conversation.nix:6,31` | input/option | `services.avatar.hostName` |
| O2 | `isNixOS` specialArg | `avatar-home-manager.nix:6,34`; `avatar-wake-home-manager.nix:6,35`; `home-manager/avatar-doctor-home-manager.nix:5,12`; `avatar-rebuild-home-manager.nix:1,10`; `avatar-remote-seat-home-manager.nix:5,12`; `avatar-voice-holder-home-manager.nix:6,33` | input/option | `services.avatar.role` (host/remote), asserted against the platform |
| O3 | Topology (host chise; machines chise, kira, rin) | `home-manager/avatar-topology.nix:1-8` | input/option | `services.avatar.topology.{host,machines}`, no owner defaults |
| O4 | `inputs.herdr` | `avatar-programs.nix:9`; used on PATH in `home-manager/avatar-floor-package.nix:13`, `avatar-wake-home-manager.nix:19`, `avatar-remote-home-manager.nix:37`, `avatar-mac-conversation.nix:89` | input/option | `services.avatar.herdrPackage` (dotfiles keeps the single herdr pin) |
| O5 | a2a client package | `avatar-programs.nix:10-12`; on PATH in `avatar-programs.nix:133`, `avatar-wake-home-manager.nix:17`, `avatar-remote-home-manager.nix:36`, `avatar-mac-conversation.nix:88` | input/option | `services.avatar.a2aPackage` |
| O6 | Python on PYTHONPATH: `agent-to-agent-communication/client/scripts`, `servants`, `hooks/runtime/common` | `avatar-programs.nix:13-17`; `scripts/wake/servant_panes.py:1` imports `a2a_cli.peer_identity`, which imports `catalog` and `interactive_session_detection` (`a2a_cli/peer_identity.py:6-7`); tests `__tests__/conftest.py:25-28` | interface | `services.avatar.peerIdentityPythonPath` now; later a CLI boundary (see PROPOSAL) |
| O7 | Servant roster `agent-harness/servants/roster.txt` | `avatar-programs.nix:126`; `home-manager/avatar-mac-conversation.nix:37`; read by `scripts/listening/transcription/cloud_transcriber.py:57`; tests `__tests__/unit/wake/test_servant_characters_file.py:5-7`, `test_wake_phrase_names.py:18` | interface | `services.avatar.servantRoster` (path); coverage of `cast/servant-characters.txt` becomes a module assertion |
| O8 | `media-speech` (`media/media-generation/speech-package.nix`) | `home-manager/avatar-control-server-tools.nix:8`; `home-manager/avatar-mac-conversation.nix:12`; spawned by `skills/avatar/scripts/control-server/speech/media-speech-client.js:12` | interface | `services.avatar.speechPackage` providing `media-speech` |
| O9 | Chrome for Testing (`browsers/chrome/chrome-for-testing-package.nix`) | `avatar-remote-home-manager.nix:17-19`; asserted textually by `__tests__/unit/seat/test_meeting_wiring.py:113` | input/option | `services.avatar.mac.browserApplication` |
| O10 | Rebuild-safe launch agents (`operating-system/launchd/rebuild-safe-launch-agent-library.nix`) | `avatar-remote-home-manager.nix:12-16,64,75` | interface | `services.avatar.mac.launchAgent` (function), default plain `launchd.agents` |
| O11 | Secrets `~/.secrets/elevenlabs-api-key`, `~/.secrets/openai-api-key` (agenix: `machine-configuration/security/secrets/public-secret-sources.nix:11,16`, `secrets/secrets.nix:33,38`) | `scripts/listening/transcription/cloud_transcriber.py:15,36`; `scripts/live/gpt_live.py:20,63`; ElevenLabs speech key read by dotfiles `media_speech_cli.py:26` | interface | `services.avatar.secrets.{elevenLabsApiKeyFile,openAiApiKeyFile}`; agenix stays in dotfiles |
| O12 | Hex (`machine-configuration/voice/hex-home-manager.nix:13-31`, `inputs.hex`) | units wait for `hex.service`: `avatar-home-manager.nix:159-160`, `avatar-wake-home-manager.nix:42,45`; runtime discovery of `~/.local/share/hex/local-api.json`: `scripts/listening/transcription/transcriber.py:15-17` | interface | `services.avatar.transcriptionService = "hex.service"`; Hex stays in dotfiles |
| O13 | Hermes CLI and profiles | `scripts/holder/hermes_turn.py:52`; profile passed by I6 | interface | `hermes` on the user profile PATH |
| O14 | `discord-voice` CLI (voice bridge `agent-harness/harnesses/clawde/discord/voice-bridge/default.nix:41`) | `scripts/conversation/discord_call.py:5`; `scripts/listening/source_capture.py:12` | interface | `discord-voice` on the user profile PATH |
| O15 | SSH host aliases and keys (`network/ssh/ssh-private-home-manager.nix`, private config) | `scripts/doctor/doctor.py:15`; `scripts/listening/remote_agent.py:10`; `scripts/wake/peer_sessions.py:14`; `scripts/conversation/remote_seat/local_seat.py:25`; `scripts/wake/mac/mac_wake_boundaries.py:23` | interface | `ssh <alias>` with BatchMode works for every topology machine |
| O16 | Hyprland window rule (I5) | `skills/avatar/scripts/browser/__tests__/unit/test_runtime.py:8-11` reads the dotfiles file | interface | class name `avatar-browser`, exported as `lib.browserWindowClass`; the assertion moves to dotfiles |
| O17 | Bats helper `repository/verification/helpers/bash-script-assertions.bash` | `__tests__/unit/avatar-stop.bats:3` | moves | port the test to pytest in the avatar repo |
| O18 | CI test tier and Python test environment (I12, I13) | see I12, I13 | moves | flake `checks` and `devShells.default` |
| O19 | Ruff per-file ignore (I11) | `.qlty/ruff.toml:17` | moves | `ruff.toml` in the avatar repo |
| O20 | `config.home.profileDirectory` on PATH | `avatar-home-manager.nix:14,88`; `avatar-wake-home-manager.nix:25`; `avatar-voice-holder-home-manager.nix:26` | moves | read inside the module (standard Home Manager) |
| O21 | nixpkgs packages and pinned downloads (whisper-cpp-vulkan, piper-tts, pipewire, blackhole, sherpa-onnx, VRM models, Piper voices, smart-turn, whisper model, FrequencyWords) | `avatar-home-manager.nix:151`; `home-manager/avatar-control-server-tools.nix:6-12`; `cast/models.nix`; `speech-voices.nix`; `home-manager/avatar-turn-model.nix`; `home-manager/avatar-transcription-model.nix`; `scripts/listening/transcription/ordinary-words.nix` | moves | `nixpkgs` input (follows dotfiles) |
| O22 | v4l2loopback device (I4) | `avatar-home-manager.nix:130` | moves | `nixosModules.default` option `services.avatar.virtualCamera` |

## 4. Standalone test evidence

Run from the extracted layout with pinned nixpkgs Python 3.12 (pytest, hypothesis, numpy, websockets), Node 22 and
coreutils only, an empty `HOME`, one batch per test directory naming every file, `--rootdir` and `-p
no:cacheprovider`.

| Run | Passed | Failed | Not collected |
| --- | ---: | ---: | --- |
| extracted tree alone | 1,050 | 5 | 9 files (`ModuleNotFoundError: a2a_cli`) plus all 13 files under `runtime/__tests__/unit/wake` (its `conftest.py` imports `floor_cli`, which reaches `a2a_cli`) |
| plus the three dotfiles Python directories of O6 on `PYTHONPATH` | 1,292 | 7 | 1 file |

The remaining 8 problems: 3 read `agent-harness/servants/roster.txt` through a path above the repository (O7), 1 reads the
dotfiles Hyprland rule (O16), and 4 call `/usr/bin/pgrep` (`scripts/conversation/remote_seat/local_seat.py:83`) or
`/usr/bin/diff` (`scripts/conversation/remote_seat/install_call_audio_drivers.py:14`), which fail on any NixOS host and in
the Nix sandbox whether extracted or not. Not run: `runtime/__tests__/unit/avatar-stop.bats` (O17) and the 3 integration
tests in `skills/avatar/__tests__/integration/`.
