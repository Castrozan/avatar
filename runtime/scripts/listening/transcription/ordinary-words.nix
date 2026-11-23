{ pkgs }:
let
  assets = import ../../../assets { inherit pkgs; };
in
builtins.concatStringsSep ":" [
  (assets.fetch "words-en")
  (assets.fetch "words-pt_br")
]
