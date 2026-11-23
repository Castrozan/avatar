{ pkgs, crane }:
let
  craneLib = crane.mkLib pkgs;
  src = craneLib.cleanCargoSource ../.;
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
