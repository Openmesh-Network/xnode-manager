{
  inputs,
  modulesPath,
  pkgs,
  lib,
  config,
  ...
}@args:
{
  imports = [
    (modulesPath + "/installer/cd-dvd/iso-image.nix")
    (import ./config.nix args)
  ];

  # EFI booting
  isoImage.makeEfiBootable = true;

  # USB booting
  isoImage.makeUsbBootable = true;

  # An installation media cannot tolerate a host config defined file
  # system layout on a fresh machine, before it has been formatted.
  swapDevices = lib.mkImageMediaOverride [ ];
  fileSystems = lib.mkImageMediaOverride config.lib.isoFileSystems;

  isoImage.isoName = lib.mkForce "xnodeos-iso-installer-${pkgs.stdenv.hostPlatform.system}.iso";
}
