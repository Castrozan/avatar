{
  description = "A local voice avatar for AI agents, as Home Manager, NixOS and nix-darwin modules";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    home-manager.url = "github:nix-community/home-manager/release-26.05";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    nix-darwin.url = "github:nix-darwin/nix-darwin/nix-darwin-26.05";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      nix-darwin,
      crane,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      packagesFor = system: nixpkgs.legacyPackages.${system};
      rustFor = system: import ./nix/rust.nix {
        pkgs = packagesFor system;
        inherit crane;
      };
    in
    {
      homeManagerModules.default = ./nix/home-manager;
      nixosModules.default = ./nix/nixos.nix;
      darwinModules.default = ./nix/nix-darwin.nix;
      skillSourceDirectory = ./skills;
      lib.browserWindowClass = "avatar-browser";

      packages = forAllSystems (
        system:
        import ./nix/packages { pkgs = packagesFor system; } // { avatar = (rustFor system).package; }
      );

      checks = forAllSystems (
        system:
        import ./nix/checks {
          inherit self home-manager nix-darwin;
          pkgs = packagesFor system;
        }
        // (rustFor system).checks
      );

      devShells = forAllSystems (system: {
        default = import ./nix/development-shell.nix { pkgs = packagesFor system; };
      });

      formatter = forAllSystems (system: (packagesFor system).nixfmt-tree);
    };
}
