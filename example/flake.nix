{
  inputs = {
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    nixpkgs.follows = "xnode-manager/nixpkgs";
  };

  nixConfig = {
    extra-substituters = [
      "https://openmesh.cachix.org"
    ];
    extra-trusted-public-keys = [
      "openmesh.cachix.org-1:du4NDeMWxcX8T5GddfuD0s/Tosl3+6b+T2+CLKHgXvQ="
    ];
  };

  outputs = inputs: {
    nixosConfigurations.container = inputs.nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit inputs;
      };
      modules = [
        inputs.xnode-manager.nixosModules.container
        inputs.xnode-manager.nixosModules.default
        inputs.xnode-manager.nixosModules.reverse-proxy
        (
          { config, ... }:
          {
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

            networking = {
              hostName = "xnode-manager";
              firewall.allowedTCPPorts = [
                config.services.xnode-manager.port
              ];
            };

            nixpkgs.hostPlatform = "x86_64-linux";
            system.stateVersion = "25.05";
          }
        )
      ];
    };
  };
}
