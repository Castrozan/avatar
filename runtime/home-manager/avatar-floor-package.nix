{
  pkgs,
  lib,
  herdr,
  unclaimedCommand,
  servantCharacters,
  a2a,
  avatarExecutable,
}:
pkgs.writeShellScriptBin "avatar-claim-floor" ''
  export PATH=${
    lib.makeBinPath (
      [
        a2a
        herdr
        unclaimedCommand
      ]
      ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
        pkgs.openssh
        pkgs.systemd
      ]
    )
  }:$PATH
  export AVATAR_SERVANT_CHARACTERS=${servantCharacters}
  exec ${avatarExecutable} claim-floor "$@"
''
