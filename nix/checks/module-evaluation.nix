{
  self,
  pkgs,
  home-manager,
  nix-darwin,
}:
let
  inherit (pkgs) lib;
  servantLines = lib.filter (line: line != "") (
    lib.splitString "\n" (builtins.readFile ../../runtime/cast/servant-characters.txt)
  );
  servantNames = map (line: lib.head (lib.splitString " | " line)) servantLines;
  rosterOf =
    names: builtins.toFile "roster.txt" (lib.concatMapStrings (name: "${name} | A Servant\n") names);
  executable = name: pkgs.writeShellScriptBin name "exit 0";
  fixture = {
    enable = true;
    topology = {
      host = "avatar-host";
      machines = [
        "avatar-host"
        "avatar-mac"
      ];
    };
    servantRoster = rosterOf servantNames;
    herdrPackage = executable "herdr";
    a2aPackage = executable "a2a";
    speechPackage = executable "media-speech";
    mac.browserApplication = "/Applications/Browser.app";
  };
  homeFor =
    settings:
    (home-manager.lib.homeManagerConfiguration {
      inherit pkgs;
      modules = [
        self.homeManagerModules.default
        {
          home = {
            username = "avatar";
            homeDirectory = if pkgs.stdenv.hostPlatform.isDarwin then "/Users/avatar" else "/home/avatar";
            stateVersion = "26.05";
          };
          services.avatar = fixture // settings;
        }
      ];
    });
  failedAssertionMessagesFor =
    settings:
    let
      interface = lib.evalModules {
        specialArgs = { inherit pkgs; };
        modules = [
          ../home-manager/options.nix
          "${pkgs.path}/nixos/modules/misc/assertions.nix"
          { services.avatar = fixture // settings; }
        ];
      };
    in
    map (assertion: assertion.message) (
      lib.filter (assertion: !assertion.assertion) interface.config.assertions
    );
  rejects =
    fragment: settings:
    lib.any (message: lib.hasInfix fragment message) (failedAssertionMessagesFor settings);
  onTheWrongPlatform = {
    hostName = if pkgs.stdenv.hostPlatform.isDarwin then "avatar-host" else "avatar-mac";
  };
  withAServantMissing = {
    hostName = if pkgs.stdenv.hostPlatform.isDarwin then "avatar-mac" else "avatar-host";
    servantRoster = rosterOf (lib.tail servantNames);
  };
  linux = rec {
    home = homeFor { hostName = "avatar-host"; };
    homeComparingRouting = homeFor {
      hostName = "avatar-host";
      rustSlices = [ "routing" ];
    };
    system = pkgs.nixos {
      imports = [ self.nixosModules.default ];
      services.avatar.virtualCamera.enable = true;
      boot.loader.grub.enable = false;
      fileSystems."/" = {
        device = "/dev/null";
        fsType = "ext4";
      };
      system.stateVersion = "26.05";
    };
    expectations = {
      "host role" = home.config.services.avatar.role == "host";
      "host units" =
        lib.attrNames (
          lib.filterAttrs (name: _: lib.hasPrefix "avatar" name) home.config.systemd.user.services
        ) == [
          "avatar-audio"
          "avatar-browser"
          "avatar-camera"
          "avatar-control-server"
          "avatar-listener"
          "avatar-live"
          "avatar-remote-seat"
          "avatar-renderer"
          "avatar-room-microphone"
          "avatar-switch-latency"
          "avatar-transcriber"
          "avatar-wake"
        ];
      "host keeps its units through a rebuild" =
        home.config.systemd.user.services.avatar-live.Unit.X-SwitchMethod == "keep-old";
      "no launch agents on the host" = home.config.launchd.agents == { };
      "no Rust slice runs by default" = home.config.services.avatar.rustSlices == [ ];
      "routing comparison is selectable" =
        homeComparingRouting.config.services.avatar.rustSlices == [ "routing" ];
      "virtual camera module" = lib.elem "v4l2loopback" system.config.boot.kernelModules;
    };
    evaluated = [
      home.activationPackage.drvPath
      homeComparingRouting.activationPackage.drvPath
      system.config.system.build.toplevel.drvPath
    ];
  };
  darwin = rec {
    home = homeFor { hostName = "avatar-mac"; };
    system = nix-darwin.lib.darwinSystem {
      modules = [
        self.darwinModules.default
        {
          services.avatar.callDevices.enable = true;
          nixpkgs.hostPlatform = pkgs.stdenv.hostPlatform.system;
          system.stateVersion = 6;
        }
      ];
    };
    expectations = {
      "remote role" = home.config.services.avatar.role == "remote";
      "launch agents keep the labels the avatar's code addresses" =
        lib.mapAttrsToList (_: agent: agent.config.Label) home.config.launchd.agents == [
          "com.dotfiles.avatar-control"
          "com.dotfiles.avatar-conversation"
          "com.dotfiles.avatar-live"
          "com.dotfiles.avatar-renderer"
          "com.dotfiles.avatar-switch-latency"
          "com.dotfiles.avatar-wake"
        ];
      "no systemd units on a Mac" = home.config.systemd.user.services == { };
      "call devices module" = lib.any (cask: cask.name == "obs") system.config.homebrew.casks;
    };
    evaluated = [
      home.activationPackage.drvPath
      system.config.system.build.toplevel.drvPath
    ];
  };
  platform = if pkgs.stdenv.hostPlatform.isDarwin then darwin else linux;
  expectations = platform.expectations // {
    "the role refuses the wrong platform" = rejects "runs on" onTheWrongPlatform;
    "a roster Servant without a character is rejected" = rejects "servantRoster" withAServantMissing;
    "the exported browser class is the one the browser runtime launches with" =
      lib.hasInfix ''BROWSER_WINDOW_CLASS = "${self.lib.browserWindowClass}"'' (
        builtins.readFile ../../skills/avatar/scripts/browser/runtime.py
      );
  };
  unmet = lib.attrNames (lib.filterAttrs (_: met: !met) expectations);
in
assert lib.assertMsg (
  unmet == [ ]
) "avatar module evaluation: unmet ${lib.concatStringsSep "; " unmet}";
pkgs.runCommand "avatar-module-evaluation" {
  evaluated = map builtins.unsafeDiscardStringContext platform.evaluated;
} "touch $out"
