{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-manager;
  xnode-manager = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.xnode-manager = {
      enable = lib.mkEnableOption "Enable Xnode Manager.";

      verbosity = lib.mkOption {
        type = lib.types.str;
        default = "warn";
        example = "info";
        description = ''
          The logging verbosity that the app should use.
        '';
      };

      dataDir = lib.mkOption {
        type = lib.types.path;
        default = "/var/lib/xnode-manager";
        example = "/var/lib/xnode-manager";
        description = ''
          The main directory to store data.
        '';
      };

      socket = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/socket";
        example = "/var/lib/xnode-manager/socket";
        description = ''
          Unix socket to interact with reverse proxy.
        '';
      };

      osDir = lib.mkOption {
        type = lib.types.path;
        default = "/etc/nixos";
        example = "/etc/nixos";
        description = ''
          The directory to store the OS configuration.
        '';
      };

      container = {
        settings = lib.mkOption {
          type = lib.types.path;
          default = "${cfg.dataDir}/containers";
          example = "/var/lib/xnode-manager/containers";
          description = ''
            The directory to store container settings.
          '';
        };

        state = lib.mkOption {
          type = lib.types.path;
          default = "/var/lib/nixos-containers";
          example = "/var/lib/nixos-containers";
          description = ''
            The directory to store container files.
          '';
        };

        profile = lib.mkOption {
          type = lib.types.path;
          default = "/nix/var/nix/profiles/per-container";
          example = "/nix/var/nix/profiles/per-container";
          description = ''
            The directory to store the container nix profile.
          '';
        };

        config = lib.mkOption {
          type = lib.types.path;
          default = "/etc/nixos-containers";
          example = "/etc/nixos-containers";
          description = ''
            The directory to store the container nspawn config.
          '';
        };

        systemd-config = lib.mkOption {
          type = lib.types.path;
          default = "/etc/systemd/system.control";
          example = "/etc/systemd/system.control";
          description = ''
            The directory to store the container systemd config.
          '';
        };
      };

      backupDir = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/backups";
        example = "/var/lib/xnode-manager/backups";
        description = ''
          The directory to store container backups.
        '';
      };

      commandstream = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/commandstream";
        example = "/var/lib/xnode-manager/commandstream";
        description = ''
          The directory to store command streams.
        '';
      };

      buildCores = lib.mkOption {
        type = lib.types.int;
        default = 0;
        example = 0;
        description = ''
          Amount of cores to use for nix builds. 0 will use all cores. See NIX_BUILD_CORES for more information.
        '';
      };

      nix = lib.mkOption {
        type = lib.types.package;
        default = pkgs.nix;
        example = pkgs.nix;
        description = ''
          nix equivalent executable to use for system building.
        '';
      };

      nixos-rebuild = lib.mkOption {
        type = lib.types.package;
        default = pkgs.nixos-rebuild;
        example = pkgs.nixos-rebuild;
        description = ''
          nixos-rebuild equivalent executable to use for OS config switching.
        '';
      };

      systemd = lib.mkOption {
        type = lib.types.package;
        default = pkgs.systemd;
        example = pkgs.systemd;
        description = ''
          systemd equivalent executable to use for container management.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.xnode-manager = {
      wantedBy = [ "multi-user.target" ];
      description = "Allow configuring and monitoring your Xnode through external platforms, such as Xnode Studio.";
      after = [ "network.target" ];
      environment = {
        RUST_LOG = cfg.verbosity;
        DATADIR = cfg.dataDir;
        SOCKET = cfg.socket;
        OSDIR = cfg.osDir;
        CONTAINERSETTINGS = cfg.container.settings;
        CONTAINERSTATE = cfg.container.state;
        CONTAINERPROFILE = cfg.container.profile;
        CONTAINERCONFIG = cfg.container.config;
        SYSTEMDCONFIG = cfg.container.systemd-config;
        BACKUPDIR = cfg.backupDir;
        COMMANDSTREAM = cfg.commandstream;
        BUILDCORES = toString cfg.buildCores;
        NIX = "${cfg.nix}/bin/";
        NIXOSREBUILD = "${cfg.nixos-rebuild}/bin/";
        SYSTEMD = "${cfg.systemd}/bin/";
        E2FSPROGS = "${pkgs.e2fsprogs}/bin/";
      };
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-manager}";
        User = "root";
        Group = "xnode-reverse-proxy"; # Grant access to unix socket
        StateDirectory = "xnode-manager";
        Restart = "always";
      };
    };

    systemd.services.start-all-containers = {
      wantedBy = [ "multi-user.target" ];
      description = "Start all NixOS containers on this host";
      after = [ "network.target" ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
      path = [
        pkgs.nixos-container
        pkgs.findutils
      ];
      script = ''
        nixos-container list | xargs -I % nixos-container start %
      '';
    };
  };
}
