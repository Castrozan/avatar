{ pkgs }:
let
  characters = import ./characters.nix { inherit pkgs; };
  voiceFields = [
    "name"
    "cloudVoice"
    "localVoice"
  ];
in
pkgs.writeText "avatar-character-voices.json" (
  builtins.toJSON (map (pkgs.lib.getAttrs voiceFields) characters)
)
