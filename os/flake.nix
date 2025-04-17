{
  description = "XnodeOS Configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    disko.url = "github:nix-community/disko/latest";
    nixos-facter-modules.url = "github:nix-community/nixos-facter-modules";
    lanzaboote.url = "github:nix-community/lanzaboote";
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
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
                    configurationLimit = 1;
                    pkiBundle = "/var/lib/sbctl";
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
                };
              }
          )
          (
            { pkgs, ... }:
            {
              boot.loader.timeout = 0; # Speed up boot by skipping selection

              environment.systemPackages = with pkgs; [
                mergerfs
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
                  dates = "daily";
                  options = "--delete-older-than 1d";
                };
              };

              users.mutableUsers = false;
              users.allowNoPasswordLogin = true;

              networking.hostName = "xnode";

              system.stateVersion = "24.11";
            }
          )
          inputs.disko.nixosModules.default
          ./disko-config.nix
          inputs.nixos-facter-modules.nixosModules.facter
          { config.facter.reportPath = ./facter.json; }
          {
            imports = [
              inputs.xnode-manager.nixosModules.default
            ];
            services.xnode-manager =
              {
                enable = true;
              }
              // (
                let
                  xnode-owner = if (builtins.pathExists ./xnode-owner) then builtins.readFile ./xnode-owner else "";
                in
                nixpkgs.lib.optionalAttrs (xnode-owner != "") {
                  owner = xnode-owner;
                }
              );
          }
          (
            let
              domain = if (builtins.pathExists ./domain) then builtins.readFile ./domain else "";
              acme-email = if (builtins.pathExists ./acme-email) then builtins.readFile ./acme-email else "";
            in
            if (domain != "" && acme-email != "") then
              # Securely expose xnode-manager
              { config, ... }:
              {
                security.acme = {
                  acceptTerms = true;
                  defaults.email = acme-email;
                  certs."xnode-manager" = {
                    listenHTTP = ":80";
                    domain = domain;
                    group = config.services.nginx.group;
                  };
                };

                services.nginx = {
                  enable = true;
                  recommendedOptimisation = true;
                  recommendedProxySettings = true;
                  recommendedTlsSettings = true;
                  proxyTimeout = "600s";

                  virtualHosts."xnode-manager" = {
                    serverName = domain;
                    addSSL = true;
                    useACMEHost = "xnode-manager";
                    listen = [
                      {
                        port = 34392;
                        addr = "0.0.0.0";
                        ssl = true;
                      }
                    ];
                    locations."/" = {
                      proxyPass = "http://localhost:34391";
                    };
                  };
                };

                # Port 80 is required to solve http-01 challenge
                networking.firewall.allowedTCPPorts = [
                  80
                  34392
                ];
              }
            else
              {
                # Xnode-manager over HTTP only (insecure)
                networking.firewall.allowedTCPPorts = [ 34391 ];
              }
          )
          (
            let
              user-passwd = if (builtins.pathExists ./user-passwd) then builtins.readFile ./user-passwd else "";
            in
            { config, ... }:
            nixpkgs.lib.optionalAttrs (user-passwd != "") {
              # No user-passwd disables password authentication entirely
              users.users.xnode = {
                initialPassword = user-passwd;
                isNormalUser = true;
                extraGroups = [
                  "networkmanager"
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
