# avatar

A local voice avatar for AI agents. A VRM character speaks for whichever agent holds the floor, listens for its Servant
name, holds conversations by voice in the room, in meetings and on Discord calls, and can stand in as a virtual camera and
microphone. The avatar host runs NixOS; Macs on nix-darwin lend it their microphone and run their own wake.

The implementation is Rust, in the `avatar` binary built from `crates/`. The renderer and the seat and wake pages are
browser JavaScript under `runtime/`, and Nix wires everything into services.

- [`runtime/README.md`](runtime/README.md) explains how the system works.

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
- `darwinModules.default` adds `services.avatar.callDevices.enable`, the BlackHole call drivers, installed by `avatar mac install-call-audio-drivers`, and OBS.
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

## Test

```sh
nix flake check -L
```

This builds the packages and the Rust workspace checks, lints, and evaluates the modules: the host role and the NixOS
module on Linux, the remote role and the nix-darwin module on macOS. CI runs it on both. `scripts/check` runs the Rust
gates alone.
