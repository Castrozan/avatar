{ self, pkgs }:
pkgs.runCommand "avatar-unit-tests"
  {
    nativeBuildInputs = [
      (import ../python-test-environment.nix { inherit pkgs; })
      pkgs.nodejs_22
    ];
  }
  ''
    export HOME="$(mktemp -d)"
    export XDG_RUNTIME_DIR="$(mktemp -d)"
    cp -r ${self} "$HOME/avatar"
    chmod -R u+w "$HOME/avatar"
    cd "$HOME/avatar"
    python -m pytest -q
    touch "$out"
  ''
