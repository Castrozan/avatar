{ pkgs }:
pkgs.buildNpmPackage {
  pname = "avatar-virtual-camera";
  version = "1.0.0";
  src = ../../skills/avatar/scripts/control-server;
  nodejs = pkgs.nodejs_22;
  npmDepsHash = "sha256-1s6vq5oGvz/tZf+9JivZvffrU2cgPHgwREkQ6eD93M8=";
  dontNpmBuild = true;
  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r . $out/
    runHook postInstall
  '';
}
