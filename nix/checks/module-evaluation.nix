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
    homeOnRust = homeFor {
      hostName = "avatar-host";
      rustSlices = [
        "routing"
        "floor"
        "talk"
        "command"
        "audio"
        "stop"
        "transcribe"
        "doctor"
        "wake"
        "holder"
        "listen"
        "start"
      ];
    };
    inherit (home.config.systemd.user) services targets;
    pictureUnits = [
      "avatar-renderer.service"
      "avatar-browser.service"
    ];
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
          "avatar-audio-recovery"
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
          "avatar-window-reveal"
        ];
      "host keeps its units through a rebuild" =
        home.config.systemd.user.services.avatar-live.Unit.X-SwitchMethod == "keep-old";
      "a hidden conversation starts no renderer or browser" =
        !(targets.avatar-conversation.Unit ? BindsTo)
        && !(lib.any (unit: lib.elem unit targets.avatar-conversation.Unit.Requires) pictureUnits);
      "the shown window brings its browser and ends with the conversation" =
        targets.avatar-window.Unit.BindsTo == [ "avatar-browser.service" ]
        && targets.avatar-window.Unit.PartOf == [ "avatar-conversation.target" ];
      "the window reveals itself again every time it starts, so a restarted browser comes back shown" =
        targets.avatar-window.Unit.Requires == [ "avatar-window-reveal.service" ]
        && lib.elem "avatar-window-reveal.service" targets.avatar-window.Unit.After
        && services.avatar-window-reveal.Unit.After == [ "avatar-browser.service" ]
        && services.avatar-window-reveal.Unit.PartOf == [ "avatar-window.target" ]
        && services.avatar-window-reveal.Service.Type == "oneshot"
        && !(services.avatar-window-reveal.Service ? RemainAfterExit);
      "the renderer and its browser stop once nothing needs them" =
        services.avatar-renderer.Unit.StopWhenUnneeded && services.avatar-browser.Unit.StopWhenUnneeded;
      "a meeting still brings the renderer, its browser and the camera" = lib.all (
        unit: lib.elem unit targets.avatar.Unit.Requires
      ) (pictureUnits ++ [ "avatar-camera.service" ]);
      "the audio server starting brings the mix and the room microphone back" =
        services.avatar-audio-recovery.Install.WantedBy == [ "pipewire-pulse.service" ]
        && services.avatar-audio-recovery.Unit.PartOf == [ "pipewire-pulse.service" ];
      "the room microphone restarts without restarting the conversation" =
        !(lib.elem "avatar-room-microphone.service" targets.avatar-conversation.Unit.Requires)
        && targets.avatar-conversation.Unit.Wants == [ "avatar-room-microphone.service" ]
        && services.avatar-room-microphone.Unit.PartOf == [ "avatar-conversation.target" ];
      "no launch agents on the host" = home.config.launchd.agents == { };
      "no Rust slice runs by default" = home.config.services.avatar.rustSlices == [ ];
      "routing comparison is selectable" =
        homeComparingRouting.config.services.avatar.rustSlices == [ "routing" ];
      "virtual camera module" = lib.elem "v4l2loopback" system.config.boot.kernelModules;
    };
    evaluated = [
      home.activationPackage.drvPath
      homeComparingRouting.activationPackage.drvPath
      homeOnRust.activationPackage.drvPath
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
