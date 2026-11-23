{ self, pkgs }:
pkgs.runCommand "avatar-lint"
  {
    nativeBuildInputs = [
      pkgs.biome
      pkgs.shellcheck
      pkgs.nixfmt
      pkgs.statix
      pkgs.deadnix
    ];
  }
  ''
    cd ${self}
    find . -type f \( -name '*.js' -o -name '*.mjs' -o -name '*.json' \) -exec biome lint --diagnostic-level=error {} +
    grep -rlE '^#!.*\b(bash|sh)$' . | xargs shellcheck --severity=warning --external-sources --source-path=SCRIPTDIR
    find . -type f -name '*.nix' -exec nixfmt --check {} +
    statix check .
    find . -type f -name '*.nix' -exec deadnix --fail {} +
    touch "$out"
  ''
