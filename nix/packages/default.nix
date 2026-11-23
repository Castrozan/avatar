{ pkgs }:
{
  renderer = import ./renderer.nix { inherit pkgs; };
}
