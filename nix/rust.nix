{ pkgs, crane }:
let
  craneLib = crane.mkLib pkgs;
  inherit (pkgs) lib;
  crateFixtureData = path: _type: builtins.match ".*/crates/.*\\.(json|log)$" path != null;
  embeddedAsset =
    path: _type:
    builtins.match ".*/(runtime/(renderer|scripts/conversation/remote_seat/page|scripts/wake/page|cast|assets)/[^/]+|skills/avatar/avatars/jarvis\\.jpeg)$" path
    != null;
  src = lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      crateFixtureData path type || embeddedAsset path type || craneLib.filterCargoSources path type;
    name = "source";
  };
  commonArguments = {
    inherit src;
    strictDeps = true;
  };
  cargoArtifacts = craneLib.buildDepsOnly commonArguments;
  withArtifacts = commonArguments // {
    inherit cargoArtifacts;
  };
in
{
  package = craneLib.buildPackage (withArtifacts // { doCheck = false; });
  checks = {
    rust-format = craneLib.cargoFmt { inherit src; };
    rust-clippy = craneLib.cargoClippy (
      withArtifacts // { cargoClippyExtraArgs = "--workspace --all-targets -- -D warnings"; }
    );
    rust-tests = craneLib.cargoNextest withArtifacts;
    rust-deny = craneLib.cargoDeny { inherit src; };
  };
}
