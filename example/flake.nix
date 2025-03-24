{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
  };

  outputs =
    {
      self,
      nixpkgs,
      xnode-manager,
      ...
    }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.container = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit xnode-manager;
        };
        modules = [
          (
            { xnode-manager, ... }:
            {
              imports = [
                xnode-manager.nixosModules.default
              ];

              boot.isContainer = true;

              nix = {
                settings = {
                  experimental-features = [
                    "nix-command"
                    "flakes"
                  ];
                };
              };

              services.xnode-manager = {
                enable = true;
                verbosity = "info";
                owner = "eth:519ce4C129a981B2CBB4C3990B1391dA24E8EbF3";
              };

              system.stateVersion = "25.05";
            }
          )
        ];
      };
    };
}
