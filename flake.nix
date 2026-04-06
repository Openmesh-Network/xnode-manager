{
  description = "Configure and monitor your Xnode";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.xz";
    systems.url = "github:nix-systems/default";
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
    {
      self,
      nixpkgs,
      systems,
    }:
    let
      # A helper that helps us define the attributes below for
      # all systems we care about.
      eachSystem =
        f:
        nixpkgs.lib.genAttrs (import systems) (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );
    in
    {
      packages = eachSystem (
        { pkgs, ... }:
        {
          default = pkgs.callPackage ./nix/package.nix { };
        }
      );

      checks = eachSystem (
        { pkgs, system, ... }:
        {
          package = self.packages.${system}.default;
          nixos-module = pkgs.callPackage ./nix/nixos-test.nix { };
        }
      );

      nixosModules = {
        default = ./nix/nixos-module.nix;
      };
    };
}
