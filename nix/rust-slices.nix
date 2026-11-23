{
  pkgs,
  lib,
  avatar,
}:
let
  enabled = slice: lib.elem slice avatar.rustSlices;
  executable = "${avatar.rustPackage}/bin/avatar";
in
{
  inherit enabled;
  execute = slice: python: if enabled slice then ''exec ${executable} ${slice} "$@"'' else python;
  shadowEnvironment =
    variable: slice:
    lib.optionalString (enabled slice) "export ${variable}=${pkgs.writeShellScript "avatar-${slice}" "exec ${executable} ${slice}"}";
}
