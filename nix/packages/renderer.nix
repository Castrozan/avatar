{ pkgs }:
let
  assets = import ../../runtime/assets { inherit pkgs; };
  three = assets.fetch "three";
  vrm = assets.fetch "three-vrm";
  characters = import ../../runtime/cast/characters.nix { inherit pkgs; };
  characterCatalog = pkgs.writeText "characters.json" (
    builtins.toJSON (map (character: { inherit (character) name description; }) characters)
  );
  linkCharacter = character: "ln -s ${character.model} $out/characters/${character.name}.vrm";
in
pkgs.runCommand "avatar-renderer" { } ''
  mkdir -p $out/vendor/three $out/vendor/vrm $out/characters
  tar xf ${three} --strip-components=1 -C $out/vendor/three
  tar xf ${vrm} --strip-components=1 -C $out/vendor/vrm
  cp ${../../runtime/renderer}/* $out/
  ${pkgs.lib.concatMapStringsSep "\n" linkCharacter characters}
  ln -s ${characterCatalog} $out/characters.json
  ln -s ${../../skills/avatar/avatars} $out/portraits
''
