{
  inputs = {
    xnodeos.url = "github:Openmesh-Network/xnodeos";
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
        inputs.xnodeos.nixosModules.container
        {
          services.xnode-container.xnode-config = ./xnode-config;
        }
        inputs.xnode-manager.nixosModules.default
        inputs.xnodeos.nixosModules.reverse-proxy
        inputs.xnode-auth.nixosModules.default
        (
          { config, lib, ... }:
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

            security.acme = {
              acceptTerms = true;
              defaults.email = "info@xnode-manager.container";
            };

            systemd.services."acme-xnode-manager.container".script = lib.mkForce ''echo "selfsigned only"'';
            services.xnode-reverse-proxy = {
              enable = true;
              rules = {
                "xnode-manager.container" = [
                  {
                    forward = "http://unix:${config.services.xnode-manager.socket}";
                  }
                ];
              };
            };

            services.xnode-auth = {
              enable = true;
              domains."xnode-manager.container".accessList."eth:519ce4c129a981b2cbb4c3990b1391da24e8ebf3" = { };
            };
          }
        )
      ];
    };
  };
}
