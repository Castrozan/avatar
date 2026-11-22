# Avatar as a standalone flake: layout, interface and cutover

The avatar's history was extracted from dotfiles `main` at `a7de46497b2bffb226320119158358c598e150fc` and published as
`github.com/Castrozan/avatar`. At extraction, dotfiles still builds and deploys the avatar from its own tree, unchanged.
`INVENTORY.md` next to this file lists every coupling cited below as `I<n>` (inbound) and `O<n>` (outbound).

## Decisions

| Decision | Choice | Reason |
| --- | --- | --- |
| Repository | `Castrozan/avatar`, public | The owner's call. Dotfiles is public and already carries the same history. |
| History scope | every current avatar file, plus every earlier name of it found by following `git log -M` renames | `git log --follow` works for every file, and no unrelated path comes along. Deleted files that no current file descends from are left out. |
| Layout until cutover | prefix renames only: `machine-configuration/media/avatar/` to `runtime/`, `agent-harness/agent-instructions/skills/workstation/avatar/` to `skills/avatar/` | Contents stay byte-identical to dotfiles, so relative paths inside each subtree still resolve and syncs merge cleanly. The root stays under the 15-entry directory limit. |
| Skill directory shape | `skills/avatar/`, exported as `skillSourceDirectory = ./skills` | Dotfiles discovers skills by `readDir` over a directory of skill directories (`skill-set-builders.nix:8-15,29-33`). A plain source path fits that shape with no import-from-derivation. |
| Skill text | moves with the code | `SKILL.md` documents the commands this repository ships, so they change together. Dotfiles already deploys instructions from a flake input (`inputs.clawde.stewardPayloadPath`). |
| README | short repository README at the root; the system documentation stays at `runtime/README.md` | The synced file is never edited here, so it never conflicts in a sync. |
| Flake shape | the clawde pattern: rev-pinned input in `repository/flake-assembly/dependencies/flake.nix` with `inputs.nixpkgs.follows`, `homeManagerModules.default`, plain-path outputs for data, `checks`, `devShells`, `formatter`, CI running `nix flake check` on Linux and macOS | This fleet already uses it, and dotfiles consumes clawde this way today. |
| Option namespace | `services.avatar` | This is Home Manager's convention for a module that owns user services. |
| Tools owned by dotfiles | package options (`herdrPackage`, `a2aPackage`, `speechPackage`), not flake inputs | Dotfiles keeps its single herdr pin and owns a2a and `media-speech`. The avatar needs the executables, not their sources. |
| Owner-specific facts | required options without defaults (`hostName`, `topology`, `servantRoster`) | A module that defaults to chise, kira and rin is not standalone. |
| Sync rule | until cutover, dotfiles stays the source of truth and this repository only adds files dotfiles does not own | Merging a new extraction can then never conflict. |
| Tests | pytest and Node in the Nix sandbox as `checks.<system>.unit-tests`; the one bats test is ported to pytest | One framework, and a sandboxed run proves there is no hidden host dependency. |
| Lint | ruff with dotfiles' selection (`E4 E7 E9 F B S`, `S603`/`S607` ignored, `S101` in tests, `S310` for `cloud_transcriber.py`), biome for JS, shellcheck, nixfmt, statix, deadnix | These are the gates the code already passes through dotfiles' qlty. Dotfiles-wide hygiene (line-count baseline, ast-grep policy) stays in dotfiles. |

## Layout

Synced code lives in exactly two prefixes: `runtime/` mirrors the dotfiles avatar directory and `skills/avatar/` mirrors
the agent skill. Everything else at the root is owned by this repository: the README, `docs/`, and later the flake, lint
configuration and CI. After cutover, when this repository becomes the source of truth, the control server and browser
runtime move from `skills/avatar/scripts/` to `runtime/control-server/` and `runtime/browser/`, so `skills/avatar/` holds
only instructions and portraits. The nix files gather in `runtime/nix/`, and the Rust routing core lands as a `routing/`
crate with its own check.

## Flake interface

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    home-manager.url = "github:nix-community/home-manager/release-26.05";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, home-manager }: {
    homeManagerModules.default = ./runtime/avatar-home-manager.nix;
    nixosModules.default = ./runtime/avatar-nixos.nix;
    darwinModules.default = ./runtime/avatar-call-devices-nix-darwin.nix;
    skillSourceDirectory = ./skills;
    lib.browserWindowClass = "avatar-browser";
    packages = forAllSystems (system: { control-server = ...; renderer = ...; });
    checks = forAllSystems (system: { unit-tests = ...; module-evaluation = ...; formatting = ...; lint = ...; });
    devShells = forAllSystems (system: { default = ...; });
    formatter = forAllSystems (system: ...);
  };
}
```

`runtime/avatar-nixos.nix` does not exist yet; step 1 of the cutover creates it from the v4l2loopback lines.
`home-manager` is only for the module-evaluation checks, which evaluate the host role on `x86_64-linux` and the remote
role on `aarch64-darwin`. Dotfiles makes it follow its own. The dev shell carries Python 3.12 with pytest, hypothesis, numpy
and websockets, plus Node 22, ruff, biome, nixfmt-rfc-style, statix, deadnix and shellcheck.

### Home Manager options (`services.avatar`)

| Option | Type and default | Replaces |
| --- | --- | --- |
| `enable` | bool, `false` | the imports in I1 and I2 |
| `role` | `"host"` or `"remote"`, default `host` when `hostName == topology.host`; asserts host is Linux and remote is Darwin | O2 `isNixOS` |
| `hostName` | str, required | O1 `hostname` specialArg |
| `topology.host`, `topology.machines` | str and list of str, required | O3 `home-manager/avatar-topology.nix` |
| `servantRoster` | path, required; asserts every roster name has a line in `cast/servant-characters.txt` | O7 |
| `herdrPackage` | package, required | O4 `inputs.herdr` |
| `a2aPackage` | package, required | O5 |
| `peerIdentityPythonPath` | list of paths, default `[ ]` | O6 PYTHONPATH reach into `agent-harness/` |
| `speechPackage` | package providing `bin/media-speech`, required | O8 |
| `transcriptionService` | str, default `"hex.service"` | O12 unit name hard-coded in four places |
| `virtualCameraDevice` | str, default `"/dev/video10"` | O22 |
| `secrets.elevenLabsApiKeyFile`, `secrets.openAiApiKeyFile` | str, default `~/.secrets/elevenlabs-api-key` and `~/.secrets/openai-api-key` | O11 paths hard-coded in Python |
| `mac.browserApplication` | str, required for the remote role | O9 Chrome for Testing import |
| `mac.launchAgent` | function to a Home Manager config, default plain `launchd.agents` | O10 rebuild-safe launch agent library |

The NixOS module adds `services.avatar.virtualCamera = { enable; videoNumber = 10; cardLabel = "Avatar Cam"; }`, which
owns the v4l2loopback lines now in chise's `configuration.nix:31-35` (I4). The nix-darwin module adds
`services.avatar.callDevices.enable` for the BlackHole drivers and the OBS cask.

Some contracts stay in dotfiles and are not options: `hermes` (O13) and `discord-voice` (O14) on the user profile PATH,
BatchMode `ssh` to each topology alias (O15), Hex's service and local API (O12), agenix-materialized key files (O11), and
the Hyprland rule for `lib.browserWindowClass` (O16), which a dotfiles check compares against the flake value.

## Tests and lint standalone

`nix flake check -L` runs everything in CI. Locally, `nix develop -c python -m pytest -p no:cacheprovider --rootdir .
<test files>` works; agent sessions name the files because a hook blocks whole-directory runs. The `unit-tests` check
runs pytest over `runtime/__tests__/unit`, `skills/avatar/__tests__/unit` and
`skills/avatar/scripts/browser/__tests__/unit`, with Node on PATH and `HOME` in a temporary directory.

Today the suite passes 1,050 tests standalone. With dotfiles' Python on `PYTHONPATH` it reaches 1,292, and 8 problems
remain (INVENTORY section 4). Making it fully standalone is step 2 of the cutover:

- O6: `servant_panes.py` imports `a2a_cli.peer_identity` inside `live_sessions()`, its only use, so `floor_cli`,
  `wake_listener` and the wake `conftest.py` import without dotfiles. Tests get a fake `a2a_cli` package under
  `runtime/__tests__/doubles/`. After cutover, the import gives way to a process boundary on the `a2a` CLI, and the
  `peerIdentityPythonPath` option is deleted.
- O7: the two roster tests read a fixture roster. The coverage rule becomes the module assertion above, which is
  evaluated in dotfiles against the real roster on every rebuild.
- O16: the browser test asserts only the class it launches with.
- The 4 tests that reach `/usr/bin/pgrep` and `/usr/bin/diff` (macOS paths the code uses on purpose) patch the command
  runner instead of executing them.
- O17: `avatar-stop.bats` becomes a pytest test.

The goal is zero failures and zero collection errors from `run-standalone-tests.sh` with nothing on `PYTHONPATH`.

## What dotfiles shrinks to

```nix
# repository/flake-assembly/dependencies/flake.nix
avatar.url = "github:Castrozan/avatar/<rev>";
avatar.inputs.nixpkgs.follows = "nixpkgs";
avatar.inputs.home-manager.follows = "home-manager";
```

```nix
# machine-configuration/media/avatar/avatar-home-manager.nix, imported by chise and the shared darwin layer
{ config, lib, pkgs, inputs, hostname, ... }:
let
  agentHarness = ../../../agent-harness;
in
{
  imports = [ inputs.avatar.homeManagerModules.default ];
  services.avatar = {
    enable = true;
    hostName = hostname;
    topology = { host = "chise"; machines = [ "chise" "kira" "rin" ]; };
    servantRoster = agentHarness + "/servants/roster.txt";
    herdrPackage = inputs.herdr.packages.${pkgs.stdenv.hostPlatform.system}.default;
    a2aPackage = import (agentHarness + "/agent-to-agent-communication/client/a2a-client-package.nix") { inherit pkgs; };
    peerIdentityPythonPath = [
      (agentHarness + "/agent-to-agent-communication/client/scripts")
      (agentHarness + "/servants")
      (agentHarness + "/hooks/runtime/common")
    ];
    speechPackage = import ../media-generation/speech-package.nix { inherit pkgs; };
    mac.browserApplication = "${import ../../browsers/chrome/chrome-for-testing-package.nix { inherit pkgs; }}/lib/Google Chrome for Testing.app";
    mac.launchAgent =
      (import ../../operating-system/launchd/rebuild-safe-launch-agent-library.nix { inherit config lib pkgs; })
      .mkRebuildSafeLaunchAgent;
  };
}
```

Chise's system imports `inputs.avatar.nixosModules.default` and sets `services.avatar.virtualCamera.enable = true` in
place of its v4l2loopback lines. The shared darwin system layer imports `inputs.avatar.darwinModules.default` and sets
`services.avatar.callDevices.enable = true` (I3). `skill-set-builders.nix` appends `inputs.avatar.skillSourceDirectory`
to its public skill directories. Deleted from dotfiles: the rest of `machine-configuration/media/avatar/`, the skill
directory, and `.qlty/ruff.toml:17`. The voice bridge (I6, I7) is untouched because it only uses commands and unit names.
The eval baseline entry for the old `SKILL.md` path (I9) reads as stale until a measurement is requested, which dotfiles
policy allows.

## Sync until cutover

The scripts live outside this repository, beside the published clone on chise in `~/repo/avatar-extraction-staging/`:
`extract.sh` and `run-standalone-tests.sh`.

1. Run `extract.sh`. It makes a fresh `--no-local` clone of `~/.dotfiles` at `main` or a
   given ref and filters it with git-filter-repo from the dotfiles-pinned nixpkgs. Before delivering, it verifies that
   the tree is blob-identical to the source and that history holds only inventory paths, then runs gitleaks and a pattern
   scan over every blob and commit message. It fetches the result into the published clone as `extraction/main` only if
   the previously synced tip is an ancestor, so published history is never rewritten. Filtering is deterministic, and the
   same input reproduces the same hashes.
2. `git merge -m "chore(avatar): sync dotfiles <source commit>" extraction/main`, then a normal push. The sync
   rule keeps this merge conflict-free, and the message records which dotfiles commit the sync reached.
3. If `extract.sh` reports rewritten history because a new rename pulled an older path in, do not force-push.
   Cherry-pick the new source commits, mapped through `work/report/commit-map.txt`.

Branches such as `avatar-addressing-decision` land in dotfiles first and arrive through the next sync.

## Cutover order

Dotfiles keeps building and deploying at every step.

1. **Make the module relocatable inside dotfiles**, as clawde did. Turn the avatar's Home Manager, NixOS and darwin files
   into option-driven modules with the interface above, and add the dotfiles wiring file that sets the options.
   `rebuild` on chise, kira and rin must produce the same units and launch agents; `avatar-doctor` passes on each.
2. **Make the test suite standalone** inside dotfiles (the list above). Run `extract.sh` and
   `run-standalone-tests.sh` until they report zero failures with nothing on `PYTHONPATH`.
3. **Add the flake in this repository** with add-only files: `flake.nix`, `ruff.toml` and CI workflows. Sync, then turn
   CI green on Linux and macOS. Dotfiles is untouched.
4. **Freeze**: land or rebase in-flight avatar branches in dotfiles, run the final sync, push, and note the tip.
5. **Switch dotfiles in one commit**: add the input pinned to that tip, import the three modules with the option
   settings above, point the skill catalog at `skillSourceDirectory`, delete the in-tree copies, and drop the ruff
   entry. Rebuild chise, kira and rin, then run `avatar-doctor` on each. Reverting that one commit restores the in-tree
   avatar.
6. **This repository becomes the source of truth**: delete the `extraction/main` ref, restructure the layout as described
   above, replace `peerIdentityPythonPath` with the a2a CLI boundary, and start the Rust routing core.

## Deviations in the implementation

The flake landed without steps 1 and 2, because dotfiles `main` was not to change before cutover and the sync owned
`runtime/` and `skills/avatar/`. These choices differ from the design above:

| Design above | Implemented | Why |
| --- | --- | --- |
| Modules are the converted `runtime/avatar-*.nix` files | Option-driven ports in `nix/`; the dotfiles-shaped originals were deleted at cutover, and the `runtime/` Nix files that need nothing from dotfiles are imported as they are | The originals reached `../../../agent-harness/...` and read `hostname`, `isNixOS` and `inputs`. |
| `transcriptionService`, `virtualCameraDevice`, `secrets.*`, `virtualCamera.videoNumber` and `cardLabel` options | Not offered; `hex.service`, `/dev/video10`, `Avatar Cam` and the `~/.secrets` key files are fixed contracts | `doctor.py`, `start.py`, the camera scripts, the renderer, `cloud_transcriber.py` and `gpt_live.py` hard-code them, so other values would be half applied. |
| Inputs `nixpkgs` and `home-manager` | Also `nix-darwin`, followed by dotfiles | The module evaluation check evaluates `darwinModules.default` in a real nix-darwin system. |
| Launch agent labels left open | `com.dotfiles.<name>` kept | `launchd_units.py` and `doctor.py` address the agents by those labels. |
| Test doubles under `runtime/__tests__/doubles/`, failing tests fixed | `tests/doubles/a2a_cli`; the roster and browser class tests read this repository's own files, and `pytest.ini` deselects 3 tests | The 3 run `/usr/bin/diff` and `/usr/bin/pgrep`, which the Linux sandbox lacks ([`CUTOVER.md`](CUTOVER.md)). |
| `checks.<system>.unit-tests` | Also `checks.<system>.integration-tests`, over `skills/avatar/__tests__/integration` | Dotfiles CI ran that tier as its own job. |
| Lint: ruff, biome, shellcheck, nixfmt, statix, deadnix | The same, but biome lints JavaScript and JSON only and shellcheck reports warnings and errors | `runtime/renderer/controls.html` has 5 biome errors and `virtual-mic.sh` 2 shellcheck style findings. |
| `skill-set-builders.nix` appends `inputs.avatar.skillSourceDirectory` | It does, and the catalog's ten callers now pass `inputs` | The catalog took only `hostname` and `pkgs`. |
| Chise's system imports `nixosModules.default` | `machine-configuration/media/avatar/avatar-virtual-camera-nixos.nix` imports it, beside `avatar-home-manager.nix` and `avatar-call-devices-nix-darwin.nix` | Dotfiles places reusable modules under their capability directory. |

## Open for the owner

- **License.** Dotfiles has none, the control server's `package.json` says MIT, and the cast models are CC0. Without a
  `LICENSE` file, the public repository grants no reuse rights.
- **Commit identities.** 16 of the 264 extracted commits carry the owner's work address as author. Those same commits
  are already public in dotfiles. Rewriting them with a mailmap would change every later hash, so it was not
  done.
