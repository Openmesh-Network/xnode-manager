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
      enable = lib.mkEnableOption "Enable the rust app";

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

      owner = lib.mkOption {
        type = lib.types.str;
        default = "eth:0000000000000000000000000000000000000000";
        example = "eth:519ce4C129a981B2CBB4C3990B1391dA24E8EbF3";
        description = ''
          The user id of the owner of this Xnode. This user has full management control.
        '';
      };

      dataDir = lib.mkOption {
        type = lib.types.str;
        default = "/var/lib/xnode-manager";
        example = "/var/lib/xnode-manager";
        description = ''
          The main directory to store data.
        '';
      };

      osDir = lib.mkOption {
        type = lib.types.str;
        default = "/etc/nixos";
        example = "/etc/nixos";
        description = ''
          The directory to store the OS configuration.
        '';
      };

      containerDir = lib.mkOption {
        type = lib.types.str;
        default = "${cfg.dataDir}/containers";
        example = "/var/lib/xnode-manage/containers";
        description = ''
          The directory to store container configurations.
        '';
      };

      backupDir = lib.mkOption {
        type = lib.types.str;
        default = "${cfg.dataDir}/backups";
        example = "/var/lib/xnode-manage/backups";
        description = ''
          The directory to store container backups.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.xnode-manager = {
      wantedBy = [ "multi-user.target" ];
      description = "Rust App.";
      after = [ "network.target" ];
      environment = {
        HOSTNAME = cfg.hostname;
        PORT = toString cfg.port;
        OWNER = cfg.owner;
        DATADIR = cfg.dataDir;
        OSDIR = cfg.osDir;
        CONTAINERDIR = cfg.containerDir;
        BACKUPDIR = cfg.backupDir;
      };
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-manager}";
        User = "root";
        Group = "root";
        CacheDirectory = "rust-app";
      };
    };

    networking.firewall.enable = false;

    systemd.services."start-all-containers" = {
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
