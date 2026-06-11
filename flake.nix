{
  inputs = {
    xnode-builders.url = "github:Openmesh-Network/xnode-builders";
  };

  nixConfig = {
    extra-substituters = [
      "https://openmesh.cachix.org"
    ];
    extra-trusted-public-keys = [
      "openmesh.cachix.org-1:du4NDeMWxcX8T5GddfuD0s/Tosl3+6b+T2+CLKHgXvQ="
    ];
  };

  outputs =
    inputs:
    inputs.xnode-builders.language.auto {
      src = ./.;
      getArgs =
        { pkgs, ... }:
        {
          extraPackageArgs = {
            crateOverrides = {
              xnode-manager = old: {
                buildInputs = (old.buildInputs or [ ]) ++ [ pkgs.acl ];
              };
            };
          };
        };
      module = {
        network = false;
        storage = false;
        user = false;
        options =
          {
            cfg,
            pkgs,
            lib,
            ...
          }:
          {
            unit = {
              alwaysOn = {
                option = {
                  type = lib.types.bool;
                  default = true;
                  example = false;
                  description = ''
                    Ensure this unit always keeps running.
                  '';
                };
                does =
                  { value, service, ... }:
                  lib.mkIf value (service {
                    startLimitIntervalSec = 0;
                    serviceConfig.Restart = "always";
                  });
              };

              runtimeDirectory = {
                enable = {
                  option = {
                    type = lib.types.bool;
                    default = true;
                    example = false;
                    description = ''
                      Create /run directory for this unit.
                    '';
                  };
                  does =
                    { value, service, ... }:
                    lib.mkIf value (service {
                      serviceConfig.RuntimeDirectory = "xnode-manager";
                    });
                };
                mode = {
                  option = {
                    type = lib.types.int;
                    default = 0700;
                    example = 0777;
                    description = ''
                      Permissions on the created runtime directory for this unit.
                    '';
                  };
                  does =
                    { value, service, ... }:
                    service {
                      serviceConfig.RuntimeDirectoryMode = value;
                    };
                };
              };

              stateDirectory = {
                enable = {
                  option = {
                    type = lib.types.bool;
                    default = true;
                    example = false;
                    description = ''
                      Create /var/lib directory for this unit.
                    '';
                  };
                  does =
                    { value, service, ... }:
                    lib.mkIf value (service {
                      serviceConfig.StateDirectory = "xnode-manager";
                    });
                };
                mode = {
                  option = {
                    type = lib.types.int;
                    default = 0700;
                    example = 0777;
                    description = ''
                      Permissions on the created state directory for this unit.
                    '';
                  };
                  does =
                    { value, service, ... }:
                    service {
                      serviceConfig.StateDirectoryMode = value;
                    };
                };
              };

              container = {
                enable = {
                  option = {
                    type = lib.types.bool;
                    default = true;
                    example = false;
                    description = ''
                      Enable container unit.
                    '';
                  };
                  does =
                    { value, ... }:
                    lib.mkIf value {
                      systemd.services."container@" = {
                        scriptArgs = "%i";
                        script = ''
                          name="$1"
                          mapfile -t args < "/var/lib/xnode-manager/host/permission/container/cli/''${name}"
                          "${cfg.systemd}/bin/systemd-nspawn" \
                            --boot \
                            --machine="''${name}.container" \
                            --slice="''${name//-/_}-container-machine.slice" \
                            --directory="${cfg.dataDir}/container/''${name}/data" \
                            "''${args[@]}"
                        '';
                        startLimitIntervalSec = 0;
                        serviceConfig.Restart = "on-failure";
                      };

                      boot.extraSystemdUnitPaths = [
                        "${cfg.dataDir}/host/permission/container/systemd"
                      ];
                    };
                };
              };

              virtual-machine = {
                enable = {
                  option = {
                    type = lib.types.bool;
                    default = true;
                    example = false;
                    description = ''
                      Enable virtual-machine unit.
                    '';
                  };
                  does =
                    { value, ... }:
                    lib.mkIf value {
                      systemd.services."virtual-machine@" = {
                        scriptArgs = "%i";
                        script = ''
                          name="$1"
                          mapfile -t args < "/var/lib/xnode-manager/host/permission/virtual-machine/cli/''${name}"
                          "${cfg.systemd}/bin/systemd-vmspawn" \
                            --machine="''${name}.virtual-machine" \
                            --slice="''${name//-/_}-virtual_machine-machine.slice" \
                            --directory="${cfg.dataDir}/virtual-machine/''${name}/data" \
                            "''${args[@]}"
                        '';
                        startLimitIntervalSec = 0;
                        serviceConfig.Restart = "on-failure";
                      };

                      boot.extraSystemdUnitPaths = [
                        "${cfg.dataDir}/host/permission/virtual-machine/systemd"
                      ];
                    };
                };
              };
            };

            verbosity = {
              option = {
                type = lib.types.str;
                default = "warn";
                example = "info";
                description = ''
                  The logging verbosity that the app should use.
                '';
              };
              does = { value, service, ... }: service { environment.RUST_LOG = value; };
            };

            dataDir = {
              option = {
                type = lib.types.path;
                default = "/var/lib/xnode-manager";
                example = "/var/lib/xnode-manager";
                description = ''
                  The main directory to store data.
                '';
              };
              does = { value, service, ... }: service { environment.DATADIR = value; };
            };

            socket = {
              option = {
                type = lib.types.path;
                default = "/run/xnode-manager/xnode-manager.sock";
                example = "/var/lib/xnode-manager/xnode-manager.sock";
                description = ''
                  Unix socket to interact with the app.
                '';
              };
              does = { value, service, ... }: service { environment.SOCKET = value; };
            };

            nix = {
              option = {
                type = lib.types.str;
                default = "/run/current-system/sw";
                example = pkgs.nix.outPath;
                description = ''
                  nix equivalent executable.
                '';
              };
              does = { value, service, ... }: service { environment.NIX = "${value}/bin/"; };
            };

            systemd = {
              option = {
                type = lib.types.str;
                default = "/run/current-system/sw";
                example = pkgs.systemd.outPath;
                description = ''
                  systemd equivalent executable.
                '';
              };
              does = { value, service, ... }: service { environment.SYSTEMD = "${value}/bin/"; };
            };

            btrfs = {
              option = {
                type = lib.types.str;
                default = pkgs.btrfs-progs.outPath;
                example = pkgs.btrfs-progs.outPath;
                description = ''
                  btrfs-progs equivalent executable.
                '';
              };
              does = { value, service, ... }: service { environment.BTRFS = "${value}/bin/"; };
            };

            find = {
              option = {
                type = lib.types.str;
                default = pkgs.findutils.outPath;
                example = pkgs.findutils.outPath;
                description = ''
                  find equivalent executable.
                '';
              };
              does = { value, service, ... }: service { environment.FIND = "${value}/bin/"; };
            };

            buildBase = {
              option = {
                type = lib.types.submodule {
                  options = {
                    container = lib.mkOption {
                      type = lib.types.str;
                      description = ''
                        NixOS configuration that will be applied on creation to build subsequent container configurations.
                      '';
                    };
                  };
                };
              };
              does = { value, service, ... }: service { environment.BUILD_BASE = builtins.toJSON value; };
            };

            defaultPermission =
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
                                type = lib.types.nullOr (
                                  lib.types.enum [
                                    "Strict"
                                    "Closed"
                                    "Auto"
                                  ]
                                );
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
                option = {
                  type = lib.types.submodule {
                    options = {
                      container = lib.mkOption {
                        type = permissionType;
                        default = { };
                        description = ''
                          Default permissions for new containers without any permissions set.
                        '';
                      };

                      virtual-machine = lib.mkOption {
                        type = permissionType;
                        default = { };
                        description = ''
                          Default permissions for new virtual machines without any permissions set.
                        '';
                      };
                    };
                  };
                  default = { };
                };
                does =
                  {
                    value,
                    service,
                    ...
                  }:
                  service { environment.DEFAULT_PERMISSION = builtins.toJSON value; };
              };

            recommendedDefaultPermission = {
              option = {
                type = lib.types.bool;
                default = true;
                example = false;
                description = ''
                  Add recommended default permissions.
                '';
              };
              does =
                { value, config, ... }:
                lib.mkIf value (config {
                  defaultPermission = {
                    container = {
                      extra_args = [
                        "--network-veth"
                        "--private-users=managed"
                        "--private-users-ownership=foreign"
                      ];
                    };
                    virtual-machine = {
                      extra_args = [
                        "--network-tap"
                      ];
                    };
                  };
                });
            };
          };
      };
    };
}
