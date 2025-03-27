{
  description = "XnodeOS Configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nixos-facter-modules.url = "github:nix-community/nixos-facter-modules";
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
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
    { nixpkgs, ... }@inputs:
    {
      nixosConfigurations.xnode = nixpkgs.lib.nixosSystem {
        specialArgs = { inherit inputs; };
        modules = [
          (
            { pkgs, ... }:
            {
              boot.loader = {
                efi = {
                  efiSysMountPoint =
                    if (builtins.pathExists ./bootpoint) then (builtins.readFile ./bootpoint) else "/boot";
                };
                grub = {
                  enable = true;
                  efiSupport = true;
                  efiInstallAsRemovable = true;
                  device = "nodev";
                };
              };

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
          ./disk-config.nix
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
              // nixpkgs.lib.optionalAttrs (builtins.pathExists ./xnode-owner) {
                owner = builtins.readFile ./xnode-owner;
              };
          }
          (
            if (builtins.pathExists ./domain && builtins.pathExists ./acme-email) then
              # Securely expose xnode-manager
              { config, ... }:
              let
                domain = builtins.readFile ./domain;
                acme-email = builtins.readFile ./acme-email;
              in
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
            { config, ... }:
            nixpkgs.lib.optionalAttrs (builtins.pathExists ./user-passwd) {
              # No user-passwd disables password authentication entirely
              users.users.xnode = {
                initialPassword = builtins.readFile ./user-passwd;
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
