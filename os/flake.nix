{
  description = "XnodeOS Configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nixos-facter-modules.url = "github:nix-community/nixos-facter-modules";
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
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
            { config, ... }:
            nixpkgs.lib.optionalAttrs (builtins.pathExists ./user-passwd) {
              # No password set disables password authentication entirely
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
