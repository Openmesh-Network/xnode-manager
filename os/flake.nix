{
  description = "XnodeOS Configuration";

  inputs = {
    disko.url = "github:nix-community/disko/latest";
    nixos-facter-modules.url = "github:nix-community/nixos-facter-modules";
    lanzaboote.url = "github:nix-community/lanzaboote";
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    nixpkgs.follows = "xnode-manager/nixpkgs";
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
                  "cache.files=off"
                  "category.create=mfs"
                  "dropcacheonclose=false"
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
                  settings.DriverQuirks.UseDefaultInterface = true;
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
                networks =
                  let
                    baseNetworkConfig = {
                      DHCP = "yes";
                      DNSSEC = "yes";
                      DNSOverTLS = "yes";
                      DNS = [
                        "1.1.1.1"
                        "8.8.8.8"
                      ];
                    };
                  in
                  {
                    "wired" = {
                      matchConfig.Name = "en*";
                      networkConfig = baseNetworkConfig;
                      dhcpV4Config.RouteMetric = 100;
                      dhcpV6Config.WithoutRA = "solicit";
                    };
                    "wireless" = {
                      matchConfig.Name = "wl*";
                      networkConfig = baseNetworkConfig;
                      dhcpV4Config.RouteMetric = 200;
                      dhcpV6Config.WithoutRA = "solicit";
                    };
                  };
              };

              services.resolved = {
                enable = true;
                llmnr = "false";
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
          (
            let
              xnode-owner = if (builtins.pathExists ./xnode-owner) then builtins.readFile ./xnode-owner else "";
            in
            { lib, ... }:
            {
              services.xnode-manager = {
                enable = true;
                owner = lib.mkIf (xnode-owner != "") xnode-owner;
              };
            }
          )
          inputs.xnode-manager.nixosModules.reverse-proxy
          (
            let
              domain = if (builtins.pathExists ./domain) then builtins.readFile ./domain else "";
              acme-email = if (builtins.pathExists ./acme-email) then builtins.readFile ./acme-email else "";
            in
            { config, lib, ... }:
            {
              security.acme = lib.mkIf (acme-email != "") {
                acceptTerms = true;
                defaults.email = acme-email;
              };

              # Securely expose xnode-manager
              services.xnode-reverse-proxy = {
                enable = true;
                rules = lib.mkIf (domain != "") {
                  "${domain}" = [
                    {
                      forward = "http://localhost:${builtins.toString config.services.xnode-manager.port}";
                    }
                  ];
                };
              };

              # Always allow xnode-manager access over HTTP
              networking.firewall.allowedTCPPorts = [ config.services.xnode-manager.port ];
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
            # START USER CONFIG
            { }
            # END USER CONFIG
          )
        ];
      };
    };
}
