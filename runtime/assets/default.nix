{ pkgs }:
let
  manifest = builtins.fromJSON (builtins.readFile ./manifest.json);
  entryNamed = name: builtins.head (builtins.filter (entry: entry.name == name) manifest.downloads);
in
{
  fetch =
    name:
    let
      entry = entryNamed name;
    in
    pkgs.fetchurl (
      {
        inherit (entry) url;
        hash = entry.integrity;
      }
      // pkgs.lib.optionalAttrs (entry ? storeName) { name = entry.storeName; }
    );
}
