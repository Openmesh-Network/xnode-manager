{
  inputs,
  modulesPath,
  pkgs,
  lib,
  ...
}@args:
{
  imports = [
    (modulesPath + "/installer/cd-dvd/installation-cd-minimal.nix")
    (import ./config.nix args)
  ];

  isoImage.isoName = lib.mkForce "xnodeos-iso-installer-${pkgs.stdenv.hostPlatform.system}.iso";
}
