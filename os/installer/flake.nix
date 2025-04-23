{
  description = "XnodeOS Kexec Installer";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    disko = { 
      url = "github:nix-community/disko/latest";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-substituters = [ "https://nix-community.cachix.org" ];
    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs =
    {
      self,
      nixpkgs,
      ...
    }@inputs:
    let
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          kexec = (pkgs.nixos [ self.nixosModules.kexec ]).config.system.build.kexecInstallerTarball;
          iso = (pkgs.nixos [ self.nixosModules.iso ]).config.system.build.isoImage;
        }
      );
      nixosModules = {
        kexec = { pkgs, ... }@args: import ./kexec.nix (args // { inherit inputs; });
        iso = { pkgs, ... }@args: import ./iso.nix (args // { inherit inputs; });
      };
    };
}
