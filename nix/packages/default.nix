{ pkgs }:
{
  virtual-camera = import ./virtual-camera.nix { inherit pkgs; };
  renderer = import ./renderer.nix { inherit pkgs; };
}
