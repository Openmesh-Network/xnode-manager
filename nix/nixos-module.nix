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

      default_permission =
        let
          permissionType = lib.types.submodule {
            options =
              let
                weight = lib.types.enum [
                  "Idle"
                  (lib.types.submodule {
                    options = {
                      Value = lib.mkOption {
                        type = lib.types.int;
                      };
                    };
                  })
                ];
              in
              {
                process = lib.mkOption {
                  type = lib.types.nullOr (
                    lib.types.submodule {
                      options = {
                        cpu = {
                          weight = lib.mkOption {
                            type = lib.types.nullOr weight;
                            default = null;
                            description = ''
                              In case there is more work than compute power, in what relative priority to allocate compute to this process. (default 100)
                            '';
                          };

                          max = lib.mkOption {
                            type = lib.types.nullOr lib.types.int;
                            default = null;
                            description = ''
                              Maximum compute power this process is allowed to use. (e.g. 100 is one core, 250 is two and a half cores)
                            '';
                          };

                          allowed_cores = lib.mkOption {
                            type = lib.types.nullOr (lib.types.listOf lib.types.int);
                            default = null;
                            description = ''
                              Specific core indexes, the process will only run on these cores.
                            '';
                          };
                        };

                        memory = {
                          max = lib.mkOption {
                            type = lib.types.nullOr lib.types.int;
                            default = null;
                            description = ''
                              Hard limit on memory this process is allowed to use in bytes.
                            '';
                          };

                          soft_max = lib.mkOption {
                            type = lib.types.nullOr lib.types.int;
                            default = null;
                            description = ''
                              Memory usage may go above the limit if unavoidable, but the processes are heavily slowed down and memory is taken away aggressively in such cases.
                            '';
                          };
                        };

                        subprocess = {
                          max = lib.mkOption {
                            type = lib.types.nullOr lib.types.int;
                            default = null;
                            description = ''
                              Maximum number of subprocesses this process is allowed to spawn.
                            '';
                          };
                        };

                        io = lib.mkOption {
                          type = lib.types.nullOr (
                            lib.types.attrsOf (
                              lib.types.submodule {
                                options = {
                                  weight = lib.mkOption {
                                    type = lib.types.nullOr weight;
                                    default = null;
                                    description = ''
                                      In case there is more work than IO, in what relative priority to allocate IO to this process. (default 100)
                                    '';
                                  };

                                  max_bandwidth = lib.mkOption {
                                    type = lib.types.nullOr (
                                      lib.types.submodule {
                                        options = {
                                          read = lib.mkOption {
                                            type = lib.types.nullOr lib.types.int;
                                            default = null;
                                          };

                                          write = lib.mkOption {
                                            type = lib.types.nullOr lib.types.int;
                                            default = null;
                                          };
                                        };
                                      }
                                    );
                                    default = null;
                                    description = ''
                                      Maximum block IO bandwidth this process is allowed to use in bytes.
                                    '';
                                  };

                                  max_iops = lib.mkOption {
                                    type = lib.types.nullOr (
                                      lib.types.submodule {
                                        options = {
                                          read = lib.mkOption {
                                            type = lib.types.nullOr lib.types.int;
                                            default = null;
                                          };

                                          write = lib.mkOption {
                                            type = lib.types.nullOr lib.types.int;
                                            default = null;
                                          };
                                        };
                                      }
                                    );
                                    default = null;
                                    description = ''
                                      Maximum block IO IOs-per-Second this process is allowed to use.
                                    '';
                                  };
                                };
                              }
                            )
                          );
                          default = null;
                        };
                      };
                    }
                  );
                  default = null;
                };

                disk = lib.mkOption {
                  type = lib.types.nullOr (
                    lib.types.submodule {
                      options = {
                        total = lib.mkOption {
                          type = lib.types.nullOr lib.types.int;
                          default = null;
                        };
                      };
                    }
                  );
                  default = null;
                };

                bind = lib.mkOption {
                  type = lib.types.nullOr (
                    lib.types.attrsOf (
                      lib.types.submodule {
                        options = {
                          path = lib.mkOption {
                            type = lib.types.nullOr lib.types.str;
                            default = null;
                          };

                          readonly = lib.mkOption {
                            type = lib.types.nullOr lib.types.bool;
                            default = null;
                          };
                        };
                      }
                    )
                  );
                  default = null;
                };

                device = lib.mkOption {
                  type = lib.types.nullOr (
                    lib.types.submodule {
                      options = {
                        policy = lib.mkOption {
                          type = lib.types.nullOr lib.types.enum [
                            "Strict"
                            "Closed"
                            "Auto"
                          ];
                          default = null;
                        };

                        allow = lib.mkOption {
                          type = lib.types.nullOr (
                            lib.types.attrsOf (
                              lib.types.submodule {
                                options = {
                                  read = lib.mkOption {
                                    type = lib.types.nullOr lib.types.bool;
                                    default = null;
                                  };

                                  write = lib.mkOption {
                                    type = lib.types.nullOr lib.types.bool;
                                    default = null;
                                  };

                                  mknod = lib.mkOption {
                                    type = lib.types.nullOr lib.types.bool;
                                    default = null;
                                  };
                                };
                              }
                            )
                          );
                          default = null;
                        };
                      };
                    }
                  );
                  default = null;
                };

                extra_args = lib.mkOption {
                  type = lib.types.nullOr (lib.types.listOf lib.types.str);
                  default = null;
                };
              };
          };
        in
        {
          container = lib.mkOption {
            type = permissionType;
            default = {
              extra_args = [
                "--network-veth"
                "--private-users=pick"
              ];
            };
            description = ''
              Default permissions for new containers without any permissions set.
            '';
          };

          virtual-machine = lib.mkOption {
            type = permissionType;
            default = {
              extra_args = [
                "--network-tap"
              ];
            };
            description = ''
              Default permissions for new virtual machines without any permissions set.
            '';
          };
        };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services = {
      xnode-manager = {
        wantedBy = [ "multi-user.target" ];
        description = "Allow configuring and monitoring your Xnode through external platforms, such as Xnode Studio.";
        environment = {
          RUST_LOG = cfg.verbosity;
          DATADIR = cfg.dataDir;
          SOCKET = cfg.socket;
          NIX = "${cfg.nix}/bin/";
          SYSTEMD = "${cfg.systemd}/bin/";
          BTRFS = "${cfg.btrfs}/bin/";
          DEFAULT_PERMISSION = builtins.toJSON cfg.default_permission;
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
        script = ''
          ${lib.getExe' cfg.systemd "systemd-nspawn"} \
            --machine="%i.container" \
            --slice="run-''${%i//-/_}-container-machine.slice" \
            --directory="${cfg.dataDir}/container/%i/data" \
            $(cat ${cfg.dataDir}/host/permission/container/cli/%i) \
            "${cfg.dataDir}/container/%i/data/init"
        '';
      };

      "virtual-machine@" = {
        script = ''
          ${lib.getExe' cfg.systemd "systemd-vmspawn"} \
            --machine="%i.virtual-machine" \
            --slice="run-''${%i//-/_}-virtual_machine-machine.slice" \
            --directory="${cfg.dataDir}/container/%i/data" \
            $(cat ${cfg.dataDir}/host/permission/virtual-machine/cli/%i)
        '';
      };
    };

    boot.extraSystemdUnitPaths = [
      "${cfg.dataDir}/host/permission/container/systemd"
      "${cfg.dataDir}/host/permission/virtual-machine/systemd"
    ];
  };
}
