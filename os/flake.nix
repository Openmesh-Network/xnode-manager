{
  description = "XnodeOS Configuration";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    xnode-manager = {
      url = "github:Openmesh-Network/xnode-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, ... }@inputs:
    {
      nixosConfigurations.xnode = nixpkgs.lib.nixosSystem {
        specialArgs = { inherit inputs; };
        modules = [
          {
            boot.loader = {
              efi = {
                efiSysMountPoint = "/boot/efi";
              };
              grub = {
                efiSupport = true;
                efiInstallAsRemovable = true;
                device = "nodev";
              };
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

            networking.hostName = "xnode";
          }
          ./configuration.nix
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
          nixpkgs.lib.optionalAttrs
          (builtins.pathExists ./user-passwd)
          {
            # No password set disables password authentication entirely
            users.users.xnode = {
              initialPassword = builtins.readFile ./user-passwd;
              isNormalUser = true;
              extraGroups = [
                "networkmanager"
                "wheel"
              ];
            };

            users.motd = null;
            services.getty = {
              autologinUser = "xnode";
              helpLine = ''\n'';
              greetingLine = ''<<< Welcome to Openmesh XnodeOS ${nixpkgs.config.system.nixos.label} (\m) - \l >>>'';
            };
          }
        ];
      };
    };
}
