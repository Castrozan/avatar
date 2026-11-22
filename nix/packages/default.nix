{ pkgs }:
{
  control-server = import ./control-server.nix { inherit pkgs; };
  renderer = import ./renderer.nix { inherit pkgs; };
}
