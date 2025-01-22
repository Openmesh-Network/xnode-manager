{ config
, pkgs
, lib
, ...
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

      openFirewall = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = ''
          Whether to open ports in the firewall for this application.
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
        OWNER = lib.toLower cfg.owner;
      };
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-manager}";
        DynamicUser = true;
        CacheDirectory = "rust-app";
      };
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ];
    };
  };
}
