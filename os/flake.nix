{
  description = "XnodeOS Configuration";

  inputs = {
    disko.url = "github:nix-community/disko/latest";
    nixos-facter-modules.url = "github:nix-community/nixos-facter-modules";
    lanzaboote.url = "github:nix-community/lanzaboote";

    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    nixpkgs.follows = "xnode-manager/nixpkgs";

    xnode-auth.url = "github:Openmesh-Network/xnode-auth";
  };

  nixConfig = {
    extra-substituters = [
      "https://openmesh.cachix.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "openmesh.cachix.org-1:du4NDeMWxcX8T5GddfuD0s/Tosl3+6b+T2+CLKHgXvQ="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs =
    { nixpkgs, ... }@inputs:
    {
      nixosConfigurations.xnode = nixpkgs.lib.nixosSystem {
        specialArgs = { inherit inputs; };
        modules = [
          (
            let
              encrypted = if (builtins.pathExists ./encrypted) then builtins.readFile ./encrypted else "";
            in
            if (encrypted == "1") then
              (
                { lib, config, ... }:
                {
                  # Full disk encryption + Secure Boot
                  imports = [
                    inputs.lanzaboote.nixosModules.lanzaboote
                  ];

                  # Use Secure Boot
                  boot.lanzaboote = {
                    enable = true;
                    enrollKeys = true;
                    pkiBundle = "/var/lib/sbctl";
                    configurationLimit = 1;
                  };

                  # Decrypt all LUKS devices unattended with Clevis (TPM2)
                  boot.initrd.availableKernelModules = [
                    "tpm_crb"
                    "tpm_tis"
                    "virtio-pci"
                  ];
                  boot.initrd.clevis.enable = true;
                  boot.initrd.clevis.devices = lib.mapAttrs (name: luksDevice: {
                    secretFile = ./clevis.jwe;
                  }) config.boot.initrd.luks.devices;
                }
              )
            else
              {
                # Normal boot (no encryption or Secure Boot)
                boot.loader.grub = {
                  enable = true;
                  efiSupport = true;
                  efiInstallAsRemovable = true;
                  device = "nodev";
                  configurationLimit = 1;
                };
              }
          )
          (
            { pkgs, ... }:
            {
              boot.loader.timeout = 0; # Speed up boot by skipping selection
              zramSwap.enable = true; # Compress memory

              environment.systemPackages = [
                pkgs.mergerfs
              ];

              # First disk is mounted as root file system, include it as data disk
              fileSystems."/mnt/disk0" = {
                device = "/mnt/disk0";
                options = [ "bind" ];
              };

              # Combine all data disks to store container data
              fileSystems."/data" = {
                fsType = "fuse.mergerfs";
                device = "/mnt/disk*";
                depends = [ "/mnt" ];
                options = [
                  # https://trapexit.github.io/mergerfs/quickstart/#configuration
                  "cache.files=auto-full"
                  "category.create=mfs"
                  "func.getattr=newest"
                  "dropcacheonclose=true"
                ];
              };

              fileSystems."/var/lib/nixos-containers" = {
                device = "/data/var/lib/nixos-containers";
                options = [ "bind" ];
              };

              nix = {
                settings = {
                  experimental-features = [
                    "nix-command"
                    "flakes"
                  ];
                  flake-registry = "";
                  accept-flake-config = true;
                };
                optimise.automatic = true;
                channel.enable = false;

                gc = {
                  automatic = true;
                  dates = "01:00";
                  randomizedDelaySec = "5h";
                  options = "--delete-old";
                };
              };

              users.mutableUsers = false;
              users.allowNoPasswordLogin = true;

              networking = {
                hostName = "xnode";
                useDHCP = false;
                useNetworkd = true;
                wireless.iwd = {
                  enable = true;
                };
                firewall = {
                  extraCommands = ''
                    iptables -A INPUT -i vz-+ -p udp -m udp --dport 67 -j ACCEPT
                  '';
                  extraStopCommands = ''
                    iptables -D INPUT -i vz-+ -p udp -m udp --dport 67 -j ACCEPT || true
                  '';
                };
              };

              systemd.network = {
                enable = true;
                wait-online = {
                  timeout = 10;
                  anyInterface = true;
                };
                networks = {
                  "wired" = {
                    matchConfig.Name = "en*";
                    networkConfig = {
                      DHCP = "yes";
                    };
                    dhcpV4Config.RouteMetric = 100;
                    dhcpV6Config.WithoutRA = "solicit";
                  };
                  "wireless" = {
                    matchConfig.Name = "wl*";
                    networkConfig = {
                      DHCP = "yes";
                    };
                    dhcpV4Config.RouteMetric = 200;
                    dhcpV6Config.WithoutRA = "solicit";
                  };
                  "80-container-vz" = {
                    matchConfig = {
                      Kind = "bridge";
                      Name = "vz-*";
                    };
                    networkConfig = {
                      Address = "192.168.0.0/16";
                      LinkLocalAddressing = "yes";
                      DHCPServer = "no";
                      IPMasquerade = "both";
                      LLDP = "yes";
                      EmitLLDP = "customer-bridge";
                      IPv6AcceptRA = "no";
                      IPv6SendRA = "yes";
                    };
                  };
                };
              };

              services.resolved.enable = false;
              services.dnsmasq = {
                enable = true;
                settings = {
                  server = [
                    "1.1.1.1"
                    "8.8.8.8"
                  ];
                  domain-needed = true;
                  bogus-priv = true;
                  no-resolv = true;
                  cache-size = 1000;
                  dhcp-range = [
                    "192.168.0.0,192.168.255.255,255.255.0.0,24h"
                  ];
                  expand-hosts = true;
                  local = "/container/";
                  domain = "container";
                };
              };
            }
          )
          (
            { config, ... }:
            let
              nixosVersion = config.system.nixos.release;
              pinnedVersion =
                if (builtins.pathExists ./state-version) then builtins.readFile ./state-version else "";
            in
            {
              system.stateVersion = if pinnedVersion != "" then pinnedVersion else nixosVersion;

              systemd.services.pin-state-version =
                let
                  nixosConfigDir = "/etc/nixos";
                in
                {
                  wantedBy = [ "multi-user.target" ];
                  description = "Pin state version to first booted NixOS version.";
                  serviceConfig = {
                    Type = "oneshot";
                  };
                  script = ''
                    if [ ! -f ${nixosConfigDir}/state-version ]; then
                      echo -n ${nixosVersion} > ${nixosConfigDir}/state-version
                    fi
                  '';
                };
            }
          )
          inputs.disko.nixosModules.default
          ./disko-config.nix
          inputs.nixos-facter-modules.nixosModules.facter
          { config.facter.reportPath = ./facter.json; }
          inputs.xnode-manager.nixosModules.default
          inputs.xnode-manager.nixosModules.reverse-proxy
          inputs.xnode-auth.nixosModules.default
          (
            let
              xnode-owner = if (builtins.pathExists ./xnode-owner) then builtins.readFile ./xnode-owner else "";
              domain = if (builtins.pathExists ./domain) then builtins.readFile ./domain else "";
              acme-email = if (builtins.pathExists ./acme-email) then builtins.readFile ./acme-email else "";
            in
            { config, lib, ... }:
            {
              services.xnode-manager = {
                enable = true;
              };

              security.acme = {
                acceptTerms = true;
                defaults.email = if (acme-email != "") then acme-email else "xnode@openmesh.network";
              };

              systemd.services."acme-manager.xnode.local".script = lib.mkForce ''echo "selfsigned only"'';
              services.xnode-reverse-proxy = {
                enable = true;
                rules = builtins.listToAttrs (
                  builtins.map (domain: {
                    name = domain;
                    value = [
                      { forward = "http://unix:${config.services.xnode-manager.socket}"; }
                    ];
                  }) ([ "manager.xnode.local" ] ++ (lib.optionals (domain != "") [ domain ]))
                );
              };

              services.xnode-auth = {
                enable = true;
                domains = lib.mkIf (xnode-owner != "") (
                  builtins.listToAttrs (
                    builtins.map (domain: {
                      name = domain;
                      value = {
                        accessList."${xnode-owner}" = { };
                      };
                    }) ([ "manager.xnode.local" ] ++ (lib.optionals (domain != "") [ domain ]))
                  )
                );
              };
            }
          )
          (
            let
              user-passwd = if (builtins.pathExists ./user-passwd) then builtins.readFile ./user-passwd else "";
            in
            { config, lib, ... }:
            lib.mkIf (user-passwd != "") {
              # No user-passwd disables password authentication entirely
              users.users.xnode = {
                initialPassword = user-passwd;
                isNormalUser = true;
                extraGroups = [
                  "wheel"
                ];
              };

              services.getty = {
                greetingLine = ''<<< Welcome to Openmesh XnodeOS ${config.system.nixos.label} (\m) - \l >>>'';
              };
            }
          )
          (
            { pkgs, ... }@args:
            {
              # START USER CONFIG

              # END USER CONFIG
            }
          )
        ];
      };
    };
}
