{ config, lib, ... }:
let
  cfg = config.services.xnode-container;
in
{
  options = {
    services.xnode-container = {
      local-resolve = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = true;
          example = false;
          description = ''
            Use container hosted resolver instead of sharing host.
          '';
        };
      };

      mDNS = {
        resolve = lib.mkOption {
          type = lib.types.bool;
          default = true;
          example = false;
          description = ''
            Resolve mDNS (using avahi).
          '';
        };

        publish = lib.mkOption {
          type = lib.types.bool;
          default = true;
          example = false;
          description = ''
            Publish mDNS (using avahi).
          '';
        };
      };
    };
  };

  config = {
    boot.isContainer = true;

    networking.useHostResolvConf = lib.mkIf cfg.local-resolve.enable false;
    services.resolved = lib.mkIf cfg.local-resolve.enable {
      enable = true;
      llmnr = "false";
      extraConfig = ''
        MulticastDNS=no
      ''; # Avahi handles mDNS
    };

    services.avahi = {
      enable = lib.mkIf (cfg.mDNS.resolve || cfg.mDNS.publish) true;
      nssmdns4 = lib.mkIf cfg.mDNS.resolve true;
      publish = lib.mkIf cfg.mDNS.publish {
        enable = true;
        addresses = true;
      };
      openFirewall = lib.mkIf cfg.mDNS.publish true;
    };
  };
}
