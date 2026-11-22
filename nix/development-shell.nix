{ pkgs }:
pkgs.mkShell {
  packages = [
    (import ./python-test-environment.nix { inherit pkgs; })
    pkgs.nodejs_22
    pkgs.ruff
    pkgs.biome
    pkgs.shellcheck
    pkgs.nixfmt
    pkgs.statix
    pkgs.deadnix
  ];
}
