{
  inputs = {
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    nixpkgs.follows = "xnode-manager/nixpkgs";

    xnode-auth.url = "github:Openmesh-Network/xnode-auth";
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
        {
          services.xnode-container.xnode-config = {
            host-platform = ./xnode-config/host-platform;
            state-version = ./xnode-config/state-version;
            hostname = ./xnode-config/hostname;
          };
        }
        inputs.xnode-manager.nixosModules.default
        inputs.xnode-auth.nixosModules.default
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
            };

            services.nginx = {
              enable = true;
              virtualHosts."xnode-manager.container" = {
                locations."/" = {
                  proxyPass = "http://127.0.0.1:${builtins.toString config.services.xnode-manager.port}";
                };
              };
            };

            services.xnode-auth = {
              enable = true;
              domains."xnode-manager.container".accessList."eth:519ce4c129a981b2cbb4c3990b1391da24e8ebf3" = { };
            };

            networking = {
              hostName = "xnode-manager";
              firewall.allowedTCPPorts = [
                80
              ];
            };
          }
        )
      ];
    };
  };
}
