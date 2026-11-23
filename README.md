# avatar

A local voice avatar for AI agents. A VRM character speaks for whichever agent holds the floor, listens for its Servant
name, holds conversations by voice in the room, in meetings and on Discord calls, and can stand in as a virtual camera and
microphone. The avatar host runs NixOS; Macs on nix-darwin lend it their microphone and run their own wake.

This repository was extracted with its history from [Castrozan/.dotfiles](https://github.com/Castrozan/.dotfiles):
`machine-configuration/media/avatar` became `runtime/`, and the `avatar` agent skill became `skills/avatar/`. It is now
the only source of the avatar. Dotfiles keeps no copy and installs it from this flake, pinned to a revision.

- [`runtime/README.md`](runtime/README.md) explains how the system works.
- [`docs/INVENTORY.md`](docs/INVENTORY.md) records what coupled the avatar to dotfiles when it was extracted.
- [`docs/PROPOSAL.md`](docs/PROPOSAL.md) sets out the flake interface and how the implementation departs from it.
- [`docs/CUTOVER.md`](docs/CUTOVER.md) records how dotfiles switched to this flake, how to roll it back, and what is still
  unverified.

## Install

The `avatar` binary is self-contained: it embeds the renderer, the seat and wake pages, the cast tables and a default
portrait, and with no environment variables set it runs from those defaults. Models and the renderer's libraries are
too large to embed, so `avatar setup` downloads them (see [Setup](#setup)).

### Install script

```sh
curl -fsSL https://raw.githubusercontent.com/Castrozan/avatar/main/install.sh | sh
```

The script detects the operating system and architecture, downloads the matching release archive, verifies its sha256,
installs `avatar` into `~/.local/bin` (`AVATAR_INSTALL_DIRECTORY` changes that) and copies the portraits into the data
directory. `AVATAR_VERSION=v0.1.0` picks a release instead of the latest.

### Release archives

Each release at [Castrozan/avatar releases](https://github.com/Castrozan/avatar/releases) carries one archive per
platform, `avatar-<target>.tar.gz`, with a matching `.sha256` file. Each holds the `avatar` binary, `LICENSE`, this
README and the `avatars/` portraits. Targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
`x86_64-apple-darwin` and `aarch64-apple-darwin`. Pushing a tag `v*` builds and publishes them
([`release.yml`](.github/workflows/release.yml)).

### Nix

Add the flake as an input and import its modules. The modules wrap the binary and point it at store paths through the
environment variables below, which override the defaults:

```nix
inputs.avatar.url = "github:Castrozan/avatar";
inputs.avatar.inputs.nixpkgs.follows = "nixpkgs";
```

- `homeManagerModules.default` sets up `services.avatar`: the host role on Linux (systemd user units) or the remote role
  on macOS (launch agents). [`nix/home-manager/options.nix`](nix/home-manager/options.nix) documents every option.
- `nixosModules.default` adds `services.avatar.virtualCamera.enable`, the `Avatar Cam` loopback device on
  `/dev/video10`.
- `darwinModules.default` adds `services.avatar.callDevices.enable`, the BlackHole call drivers and OBS.
- `skillSourceDirectory` is the agent skill directory, and `lib.browserWindowClass` is the window class a compositor
  should hide.

The machine also has to provide what the avatar calls but does not own: a `hex.service` user unit serving local
transcription, `hermes` and `discord-voice` on the profile PATH, `ssh <machine>` in BatchMode for every topology machine,
and the API keys in `~/.secrets/elevenlabs-api-key` and `~/.secrets/openai-api-key`.

`servantRoster` and [`runtime/cast/servant-characters.txt`](runtime/cast/servant-characters.txt) must list the same
Servants in the same order, and evaluation fails until they do. So adding, removing or reordering a Servant in the
consumer's roster needs a matching line here and a new pin.

### Skill as a plugin

The repository root is also an agent plugin carrying the `avatar` skill. In Claude Code: `/plugin marketplace add
Castrozan/avatar`, then `/plugin install avatar@avatar`. Codex reads the same marketplace: `codex plugin marketplace add
Castrozan/avatar`, then `codex plugin add avatar@avatar`. [`plugin.json`](plugin.json) makes the root an
[Agent Plugins](https://agent-plugins.org/specification) package, which spec loaders such as Pi's and Hermes' read from
its directory. A Nix consumer can register the pinned flake source as a local marketplace instead, so the skill and the
runtime move with one pin. The plugin carries the instructions only: the commands the skill calls, such as
`avatar-talk`, still come from the modules above.

## Setup

```sh
avatar setup
```

downloads everything the binary does not embed into the data directory: the voice activity, turn and speaker models,
the transcription model for a local `whisper-server`, the Piper speech voices, the ordinary-word frequency lists, the
renderer's `three` and `three-vrm` libraries and the character models. Every file is checked against the digest pinned
in [`runtime/assets/manifest.json`](runtime/assets/manifest.json) and is replaced only when it verifies, so an
interrupted run leaves nothing half written and a repeated run downloads nothing that is already present. `--manifest
FILE` reads another manifest.

The data directory is `$XDG_DATA_HOME/avatar` (`~/.local/share/avatar`) on Linux and `~/Library/Application
Support/avatar` on macOS; `AVATAR_DATA_DIRECTORY` replaces it. The Nix modules read the same manifest, so a Nix install
and `avatar setup` fetch identical bytes.

Each asset resolves in the same order: its environment variable, then the data directory.

| Variable | Default |
| --- | --- |
| `AVATAR_RENDERER_ROOT` | embedded renderer written to `<data>/renderer`, with the downloaded libraries and characters |
| `AVATAR_SEAT_PAGE`, `AVATAR_WAKE_PAGE` | embedded pages written to `<data>/seat-page` and `<data>/wake-page` |
| `AVATAR_CHARACTER_VOICES`, `AVATAR_SERVANT_CHARACTERS` | embedded cast tables written to `<data>/cast` |
| `AVATAR_ORDINARY_WORDS` | `<data>/words/*.txt` |
| `AVATAR_TURN_MODEL`, `AVATAR_VOICE_ACTIVITY_MODEL`, `AVATAR_SPEAKER_MODEL` | `<data>/models/` (after `avatar setup`) |
| `AVATAR_TTS_VOICE_DIRECTORY` | `<data>/voices` |

The portraits beyond the embedded `jarvis.jpeg` are not in the binary: the install script and the release archives put
them in `<data>/renderer/portraits`.

## Platform support

| Platform | Status |
| --- | --- |
| Linux x86_64, aarch64 | Release binaries and the Nix modules. The host role runs here. |
| macOS x86_64, aarch64 | Release binaries and the Nix modules. The remote role runs here. |
| Windows | Not supported. |

Windows does not build, and a port is a project of its own, not a flag. `cargo check --workspace --target
x86_64-pc-windows-gnu` fails in six library crates (`avatar-audio-graph`, `avatar-control`, `avatar-doctor`,
`avatar-floor`, `avatar-transcript`, `avatar-wake-log`) before `avatar-cli` is reached, and 44 source files outside the
tests use Unix-only APIs:

- `std::os::unix` in 22 files: `PermissionsExt` file modes, `OpenOptionsExt::mode` and `DirBuilderExt::mode` on the state
  and lock files, `symlink`, `CommandExt::process_group` for the owned browser and `ExitStatusExt`.
- The `nix` crate in 17 files: `nix::unistd` and `nix::sys` (users, hostnames, signals, process groups), which do not
  exist on Windows.
- `tokio::signal::unix` for SIGINT and SIGTERM handling in 5 files, `flock` based locks in 4 and `/proc` in 2.
- Runtime paths and tools with no Windows counterpart in this design: `/run/user/<uid>`, `/dev/video*`, `systemctl`,
  `pactl` and `parec`, `hyprctl` and `herdr`.

The binary's own asset code (`avatar-assets`: the embedded files, the data directory and `avatar setup`) is portable and
compiles for that target.

## Test

```sh
nix flake check -L
```

This builds the packages and the Rust workspace checks, lints, and evaluates the modules: the host role and the NixOS
module on Linux, the remote role and the nix-darwin module on macOS. CI runs it on both. `scripts/check` runs the Rust
gates alone.
