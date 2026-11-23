{ pkgs, avatarExecutable }:
pkgs.writeShellScriptBin "avatar-switch-latency" ''
  exec ${avatarExecutable} switch-latency "$@"
''
