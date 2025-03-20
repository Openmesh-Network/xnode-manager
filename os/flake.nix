{
  description = "XnodeOS Configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
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

              # Combine all disk file systems to store container data
              fileSystems."/var/lib/nixos-containers" = {
                fsType = "fuse.mergerfs";
                device = "/mnt/disk*";
                options = [
                  "cache.files=partial"
                  "dropcacheonclose=true"
                  "category.create=mfs"
                ];
              };

              # First disk is mounted as root file system, create a folder to include it
              systemd.tmpfiles.rules = [
                "d /mnt/disk0"
              ];

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
          ./hardware-configuration.nix
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
