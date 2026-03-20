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
        default = "/run/xnode-manager/.socket";
        example = "/var/lib/xnode-manager/.socket";
        description = ''
          Unix socket to interact with the app.
        '';
      };

      nix = lib.mkOption {
        type = lib.types.package;
        default = pkgs.nix;
        example = pkgs.nix;
        description = ''
          nix equivalent executable.
        '';
      };

      systemd = lib.mkOption {
        type = lib.types.package;
        default = pkgs.systemd;
        example = pkgs.systemd;
        description = ''
          systemd equivalent executable.
        '';
      };

      btrfs = lib.mkOption {
        type = lib.types.package;
        default = pkgs.btrfs-progs;
        example = pkgs.btrfs-progs;
        description = ''
          btrfs-progs equivalent executable.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services = {
      xnode-manager = {
        wantedBy = [ "multi-user.target" ];
        description = "Allow configuring and monitoring your Xnode through external platforms, such as Xnode Studio.";
        after = [ "network.target" ];
        environment = {
          RUST_LOG = cfg.verbosity;
          DATADIR = cfg.dataDir;
          SOCKET = cfg.socket;
          NIX = "${cfg.nix}/bin/";
          SYSTEMD = "${cfg.systemd}/bin/";
          BTRFS = "${cfg.btrfs}/bin/";
        };
        startLimitIntervalSec = 0;
        serviceConfig = {
          ExecStart = "${lib.getExe xnode-manager}";
          StateDirectory = "xnode-manager";
          RuntimeDirectory = "xnode-manager";
          Restart = "always";
        };
      };

      "container@" = {
        description = "Container %i";
        serviceConfig = {
          ExecReload = pkgs.writeScript "reload-container" ''
            #! ${pkgs.runtimeShell} -e
            ${cfg.systemd}/bin/systemd-run \
              --wait --quiet --collect \
              --machine="%i.container" \
              --unit="apply.service" \
              /result/bin/switch-to-configuration test
          '';
        };
        script = ''
          ${cfg.systemd}/bin/systemd-nspawn \
            --machine="%i.container" \
            --slice="run-''${%i//-/_}-container-machine.slice" \
            --directory="/var/lib/nixos-containers/%i" \
            $(cat ${cfg.dataDir}/host/permission/container/cli/%i) \
            "${cfg.dataDir}/container/%i/data/init"
        '';
      };

      "virtual-machine@" = {
        description = "Virtual Machine %i";
        serviceConfig = {
          ExecReload = pkgs.writeScript "reload-virtual-machine" ''
            #! ${pkgs.runtimeShell} -e
            ${cfg.systemd}/bin/systemd-run \
              --wait --quiet --collect \
              --machine="%i.virtual-machine" \
              --unit="apply.service" \
              /result/bin/switch-to-configuration test
          '';
        };
        script = ''
          ${cfg.systemd}/bin/systemd-vmspawn \
            --machine="%i.virtual-machine" \
            --slice="run-''${%i//-/_}-virtual_machine-machine.slice" \
            --directory="/var/lib/nixos-containers/%i" \
            $(cat ${cfg.dataDir}/host/permission/virtual-machine/cli/%i) \
            "${cfg.dataDir}/virtual-machine/%i/data/init"
        '';
      };
    };
  };
}
