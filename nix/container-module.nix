{
  inputs,
  config,
  lib,
  ...
}:
let
  cfg = config.services.xnode-container;
in
{
  options = {
    services.xnode-container = {
      xnode-config = {
        host-platform = lib.mkOption {
          type = lib.types.path;
          example = ./xnode-config/host-platform;
          description = ''
            Container host platform.
          '';
        };

        state-version = lib.mkOption {
          type = lib.types.path;
          example = ./xnode-config/state-version;
          description = ''
            Container state version.
          '';
        };

        hostname = lib.mkOption {
          type = lib.types.path;
          example = ./xnode-config/hostname;
          description = ''
            Container hostname.
          '';
        };
      };

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
    nixpkgs.hostPlatform =
      if (builtins.pathExists cfg.xnode-config.host-platform) then
        builtins.readFile cfg.xnode-config.host-platform
      else
        "x86_64-linux";
    system.stateVersion =
      if (builtins.pathExists cfg.xnode-config.state-version) then
        builtins.readFile cfg.xnode-config.state-version
      else
        config.system.nixos.release;
    systemd.services.pin-state-version = {
      wantedBy = [ "multi-user.target" ];
      description = "Pin state version to first booted NixOS version.";
      serviceConfig = {
        Type = "oneshot";
      };
      script = ''
        if [ ! -f /xnode-config/state-version ]; then
          echo -n ${config.system.nixos.release} > /xnode-config/state-version
        fi
      '';
    };
    networking.hostName = lib.mkIf (builtins.pathExists cfg.xnode-config.hostname) (
      builtins.readFile cfg.xnode-config.hostname
    );

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
