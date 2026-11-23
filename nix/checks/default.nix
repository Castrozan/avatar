{
  self,
  pkgs,
  home-manager,
  nix-darwin,
}:
let
  pythonTests = import ./python-tests.nix { inherit self pkgs; };
in
import ../packages { inherit pkgs; }
// {
  unit-tests = pythonTests "unit-tests" [ ];
  lint = import ./lint.nix { inherit self pkgs; };
  module-evaluation = import ./module-evaluation.nix {
    inherit
      self
      pkgs
      home-manager
      nix-darwin
      ;
  };
}
