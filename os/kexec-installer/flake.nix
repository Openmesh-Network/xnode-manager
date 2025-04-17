{
  description = "XnodeOS Kexec Installer";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    disko.url = "github:nix-community/disko/latest";
    nixos-facter.url = "github:nix-community/nixos-facter";
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
      packages = forAllSystems (system: {
        default =
          (nixpkgs.legacyPackages.${system}.nixos [ self.nixosModules.default ])
          .config.system.build.kexecInstallerTarball;
      });
      nixosModules = {
        default = import ./kexec.nix inputs;
      };
    };
}
