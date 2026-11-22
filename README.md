# avatar

A local voice avatar for AI agents. A VRM character speaks for whichever agent holds the floor, listens for its Servant
name, holds conversations by voice in the room, in meetings and on Discord calls, and can stand in as a virtual camera and
microphone. The avatar host runs NixOS; Macs on nix-darwin lend it their microphone and run their own wake.

This repository was extracted with its history from [Castrozan/.dotfiles](https://github.com/Castrozan/.dotfiles):
`machine-configuration/media/avatar` became `runtime/`, and the `avatar` agent skill became `skills/avatar/`. For now
the dotfiles repository is still the source of truth and still builds and deploys the avatar from its own copy. Changes
land there first and are synced here.

- [`runtime/README.md`](runtime/README.md) explains how the system works.
- [`docs/INVENTORY.md`](docs/INVENTORY.md) records what coupled the avatar to dotfiles when it was extracted.
- [`docs/PROPOSAL.md`](docs/PROPOSAL.md) sets out the flake interface and the cutover that makes this repository the
  source.
- [`docs/CUTOVER.md`](docs/CUTOVER.md) lists what remains before dotfiles installs the avatar from this flake.

## Install

Add the flake as an input and import its modules:

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

## Test

```sh
nix flake check -L
```

This builds the packages and runs the unit tests in the Nix sandbox, lint, and an evaluation of the modules: the host
role and the NixOS module on Linux, the remote role and the nix-darwin module on macOS. CI runs it on both. To run only
the unit tests, build `.#checks.x86_64-linux.unit-tests` (or `aarch64-darwin`); [`pytest.ini`](pytest.ini) holds the
test selection.
