{
  self,
  pkgs,
  home-manager,
  nix-darwin,
}:
import ../packages { inherit pkgs; }
// {
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
