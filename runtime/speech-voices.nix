{ pkgs }:
let
  assets = import ./assets { inherit pkgs; };
  linkVoice = name: ''
    ln -s ${assets.fetch "voice-${name}"} $out/${name}.onnx
    ln -s ${assets.fetch "voice-${name}-configuration"} $out/${name}.onnx.json
  '';
in
pkgs.runCommand "avatar-speech-voices" { } ''
  mkdir $out
  ${linkVoice "pt_BR-faber-medium"}
  ${linkVoice "en_US-lessac-medium"}
''
