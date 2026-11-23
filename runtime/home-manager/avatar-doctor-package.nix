{
  pkgs,
  lib,
  commands,
  arguments,
  execute ? (python: python),
}:
pkgs.writeShellScriptBin "avatar-doctor" ''
  export PATH=${lib.makeBinPath commands}:$PATH
  ${execute ''exec ${pkgs.python312}/bin/python3 ${../scripts/doctor}/doctor.py ${arguments} "$@"''}
''
