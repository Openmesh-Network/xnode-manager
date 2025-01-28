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

            networking.hostName = "Xnode";
          }
          ./configuration.nix
          {
            imports = [
              inputs.xnode-manager.nixosModules.default
            ];
            services.xnode-manager.enable = true;
            services.xnode-manager.owner = "eth:0000000000000000000000000000000000000000";
          }
        ];
      };
    };
}
