{
  pkgs,
  lib,
  commands,
  avatarExecutable,
  arguments ? "",
}:
pkgs.writeShellScriptBin "avatar-doctor" ''
  export PATH=${lib.makeBinPath commands}:$PATH
  exec ${avatarExecutable} doctor ${arguments} "$@"
''
