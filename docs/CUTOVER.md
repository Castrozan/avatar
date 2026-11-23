# Cutover: dotfiles installs the avatar from this flake

## What changed

On 2026-11-22 dotfiles stopped carrying its own copy of the avatar. Its `main` pins this repository by revision in
`repository/flake-assembly/dependencies/flake.nix`, with `nixpkgs`, `home-manager` and `nix-darwin` following dotfiles,
the way it pins clawde. The cutover commits run from `3c3c61776` to `fb8b187c9`; the first pin that every host runs is
`2a4af9e`.

- Dotfiles keeps only its wiring, three files in `machine-configuration/media/avatar/`:
  - `avatar-home-manager.nix` sets `services.avatar` for chise and both Macs.
  - `avatar-virtual-camera-nixos.nix` turns on the `Avatar Cam` loopback.
  - `avatar-call-devices-nix-darwin.nix` turns on the call devices.
- The in-tree runtime and the `avatar` skill were deleted.
- The skill catalog reads `skillSourceDirectory`, and a dotfiles check matches the Hyprland rule against
  `lib.browserWindowClass`.
- The extraction scaffolding and its sync script were deleted, so this repository is the only source.

Before the switch, the evaluation showed nothing would move:

- Every avatar systemd unit, unit file, target, timer, launch agent, activation entry, package, kernel module and
  modprobe line on chise, kira and rin evaluated identical to dotfiles `7c86a78b9`, the last commit with the in-tree
  copy.
- On chise, the 21 live avatar unit files and the 13 avatar commands on `PATH` matched that evaluation.

## Changing the avatar

1. Change it here and push. Wait for CI to pass on Linux and macOS.
2. In dotfiles, set the new revision in `repository/flake-assembly/dependencies/flake.nix`. Then run
   `nix flake update dependencies/avatar` and commit both files.
3. Rebuild each machine.

`servantRoster` must list the same Servants as `runtime/cast/servant-characters.txt`, in the same order. Before a dotfiles
roster change, add the matching line here and update the pin.

## Rollback

- To return to an earlier avatar, pin an earlier revision of this repository and rebuild. Every commit since `98f1ec9`
  carries the flake.
- To restore the in-tree copy, revert the dotfiles cutover commits `3c3c61776..fb8b187c9` and rebuild chise, kira and
  rin. That copy stops at dotfiles `7c86a78b9` and lacks every change made here since.

## Not verified

- **macOS.** Kira and rin have not been rebuilt since the cutover. Each needs `rebuild`, then `avatar-doctor`.
  - Their complete systems cannot be evaluated on Linux, before or after the cutover. Both need an `aarch64-darwin`
    import-from-derivation, `cabal2nix-cachix`.
  - The avatar was compared attribute by attribute instead: launch agents, activation entries, packages, the call
    driver activation line and the casks.
  - The macOS CI job builds the packages, runs the tests, and evaluates the remote role and `darwinModules.default` in
    a fixture system.
- **Tests set aside.** `pytest.ini` deselects 3 tests (4 cases) that run `/usr/bin/diff` and `/usr/bin/pgrep`, which
  the Linux sandbox lacks:
  - `test_call_audio_drivers.py::test_drivers_already_in_place_leave_the_audio_server_alone`
  - `test_call_audio_drivers.py::test_a_changed_driver_replaces_the_installed_one`
  - `test_remote_talk_seat_rule.py::test_a_meeting_seat_is_found_by_the_holder_told_it_holds_a_meeting`

  They should patch the command runner.
- **Timing under load.** Each of these failed once in a sandbox run on a loaded machine and passed when rerun:
  - `test_speech_output_default.py::test_a_seat_held_by_a_machine_is_attached`, against the 300 ms seat health timeout
    in `runtime/scripts/command.js`.
  - `test_live_voice.py::test_the_live_voice_is_whatever_the_table_says_for_the_assigned_character`, against the
    Hypothesis 200 ms deadline.

## Next in this repository

- Fix the 5 `useButtonType` errors in `runtime/renderer/controls.html` and the 2 SC2181 findings in
  `skills/avatar/scripts/control-server/virtual-mic.sh`. Then lint HTML with biome and run shellcheck at its default
  severity.
- Replace `peerIdentityPythonPath` with a process boundary on the `a2a` CLI, and replace the `tests/doubles/a2a_cli`
  double with it.
- Restructure the layout and start the Rust routing core, as [`PROPOSAL.md`](PROPOSAL.md) describes.
