{ pkgs }:
let
  assets = import ../assets { inherit pkgs; };
  withModelFile = character: character // { model = assets.fetch "character-${character.model}"; };
in
map withModelFile (builtins.fromJSON (builtins.readFile ./characters.json))
