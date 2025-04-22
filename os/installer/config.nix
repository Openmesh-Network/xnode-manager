{
  inputs,
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}:
{
  system.stateVersion = config.system.nixos.release;

  nix.settings = {
    extra-experimental-features = [
      "nix-command"
      "flakes"
    ];
    accept-flake-config = true;
  };

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
      pkgs.libuuid
      pkgs.jq
      pkgs.curl
      pkgs.nix
      pkgs.nixos-install
      inputs.disko.packages.${pkgs.system}.default
      inputs.nixos-facter.packages.${pkgs.system}.default
      pkgs.sbctl
      pkgs.clevis
    ];
    script = lib.readFile ./install.sh;
  };
}
