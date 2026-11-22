{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib) mkOption types;
  cfg = config.services.avatar;
  namesIn =
    file:
    map (line: lib.head (lib.splitString " | " line)) (
      lib.filter (line: line != "") (lib.splitString "\n" (builtins.readFile file))
    );
  defaultLaunchAgent =
    {
      name,
      label,
      package,
      executableName,
      serviceConfig,
      ...
    }:
    {
      launchd.agents.${name} = {
        enable = true;
        config = serviceConfig // {
          Label = label;
          ProgramArguments = [ "${package}/bin/${executableName}" ];
        };
      };
    };
in
{
  options.services.avatar = {
    enable = lib.mkEnableOption "the local voice avatar";

    role = mkOption {
      type = types.enum [
        "host"
        "remote"
      ];
      default = if cfg.hostName == cfg.topology.host then "host" else "remote";
      defaultText = lib.literalExpression ''if hostName == topology.host then "host" else "remote"'';
      description = "The host role runs the avatar on Linux with systemd user units; the remote role lends a Mac's microphone and runs its own wake through launchd agents.";
    };

    hostName = mkOption {
      type = types.str;
      description = "This machine's name in the topology, also the ssh alias its peers use to reach it.";
    };

    topology = {
      host = mkOption {
        type = types.str;
        description = "The machine that runs the host role.";
      };
      machines = mkOption {
        type = types.listOf types.str;
        description = "Every machine taking part, the host included; each must answer `ssh <name>` in BatchMode.";
      };
    };

    servantRoster = mkOption {
      type = types.path;
      description = "Servant roster, one `Name | description` line per Servant; every name needs a line in the avatar's cast/servant-characters.txt.";
    };

    herdrPackage = mkOption {
      type = types.package;
      description = "herdr, put on the PATH of the floor, wake and live voice programs.";
    };

    a2aPackage = mkOption {
      type = types.package;
      description = "The agent-to-agent client providing `bin/a2a`.";
    };

    peerIdentityPythonPath = mkOption {
      type = types.listOf types.path;
      default = [ ];
      description = "Directories that make the Python module `a2a_cli.peer_identity` importable for the floor and wake programs.";
    };

    speechPackage = mkOption {
      type = types.package;
      description = "Package providing `bin/media-speech`, the control server's speech synthesizer.";
    };

    mac = {
      browserApplication = mkOption {
        type = types.str;
        description = "Path to the macOS application bundle that opens the remote role's browser pages.";
      };
      launchAgent = mkOption {
        type = types.functionTo types.attrs;
        default = defaultLaunchAgent;
        defaultText = lib.literalMD "a plain `launchd.agents.<name>` entry running the package's executable";
        description = "Function from `{ name, label, package, executableName, serviceConfig, restartWhenPackageChanges ? false }` to a Home Manager configuration that sets only `home` and `launchd`.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.role == "host" -> pkgs.stdenv.hostPlatform.isLinux;
        message = "services.avatar: the host role runs on Linux only.";
      }
      {
        assertion = cfg.role == "remote" -> pkgs.stdenv.hostPlatform.isDarwin;
        message = "services.avatar: the remote role runs on macOS only.";
      }
      {
        assertion =
          lib.allUnique (namesIn cfg.servantRoster)
          && namesIn cfg.servantRoster == namesIn ../../runtime/cast/servant-characters.txt;
        message = "services.avatar: every Servant in servantRoster needs exactly one line, in roster order, in cast/servant-characters.txt.";
      }
    ];
  };
}
