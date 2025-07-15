{
  inputs,
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}:
{
  nix.settings = {
    extra-experimental-features = [
      "nix-command"
      "flakes"
    ];
    flake-registry = "";
    accept-flake-config = true;
  };
  nix.channel.enable = false;

  boot.loader.timeout = lib.mkForce 0;
  services.getty.autologinUser = lib.mkForce "root";
  zramSwap.enable = true;

  systemd.services.install-xnodeos = {
    wantedBy = [ "multi-user.target" ];
    description = "Install XnodeOS.";
    wants = [ "network-online.target" ];
    after = [ "network-online.target" ];
    serviceConfig = {
      Type = "oneshot";
      User = "root";
      Group = "root";
      RemainAfterExit = true;
    };
    path = [
      pkgs.util-linuxMinimal
      pkgs.jq
      pkgs.curl
      pkgs.nix
      pkgs.nixos-install
      inputs.disko.packages.${pkgs.system}.default
      pkgs.nixos-facter
      pkgs.sbctl
      pkgs.clevis
    ];
    script = lib.readFile ./install.sh;
  };

  system.stateVersion = config.system.nixos.release;

  # Reduce closure size (https://github.com/nix-community/nixos-images/blob/main/nix/noninteractive.nix)
  nix.registry = lib.mkForce { };
  environment.defaultPackages = lib.mkForce [ ];
  system.extraDependencies = lib.mkForce [ ];

  # # Disable unused nixos tools
  system.disableInstallerTools = true;

  # # Disable documentation
  documentation.enable = false;
  documentation.man.enable = false;
  documentation.nixos.enable = false;
  documentation.doc.enable = false;

  # # Disable unused programs
  programs.nano.enable = false;
  security.sudo.enable = false;

  services.dbus.implementation = "broker";
}
