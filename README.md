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
