{ pkgs }:
pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.clippy
    pkgs.rustfmt
    pkgs.cargo-nextest
    pkgs.cargo-deny
    pkgs.ast-grep
    pkgs.ruff
    pkgs.biome
    pkgs.shellcheck
    pkgs.nixfmt
    pkgs.statix
    pkgs.deadnix
  ];
}
