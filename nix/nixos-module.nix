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

      hostname = lib.mkOption {
        type = lib.types.str;
        default = "0.0.0.0";
        example = "127.0.0.1";
        description = ''
          The hostname under which the app should be accessible.
        '';
      };

      port = lib.mkOption {
        type = lib.types.port;
        default = 34391;
        example = 34391;
        description = ''
          The port under which the app should be accessible.
        '';
      };

      verbosity = lib.mkOption {
        type = lib.types.str;
        default = "warn";
        example = "info";
        description = ''
          The logging verbosity that the app should use.
        '';
      };

      owner = lib.mkOption {
        type = lib.types.str;
        default = "eth:0000000000000000000000000000000000000000";
        example = "eth:519ce4C129a981B2CBB4C3990B1391dA24E8EbF3";
        description = ''
          The user id of the owner of this Xnode. This user has full management control.
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

      osDir = lib.mkOption {
        type = lib.types.path;
        default = "/etc/nixos";
        example = "/etc/nixos";
        description = ''
          The directory to store the OS configuration.
        '';
      };

      authDir = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/auth";
        example = "/var/lib/xnode-manager/auth";
        description = ''
          The directory to store authentication information.
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
        HOSTNAME = cfg.hostname;
        PORT = toString cfg.port;
        RUST_LOG = cfg.verbosity;
        OWNER = cfg.owner;
        DATADIR = cfg.dataDir;
        OSDIR = cfg.osDir;
        AUTHDIR = cfg.authDir;
        CONTAINERSETTINGS = cfg.container.settings;
        CONTAINERSTATE = cfg.container.state;
        CONTAINERPROFILE = cfg.container.profile;
        CONTAINERCONFIG = cfg.container.config;
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
        Group = "root";
        CacheDirectory = "rust-app";
        Restart = "on-failure";
      };
    };

    systemd.services.start-all-containers = {
      wantedBy = [ "network.target" ];
      description = "Start all NixOS containers on this host";
      path = [
        pkgs.nixos-container
        pkgs.findutils
      ];
      script = ''
        nixos-container list | xargs -I % nixos-container start %
      '';

      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
    };
  };
}
