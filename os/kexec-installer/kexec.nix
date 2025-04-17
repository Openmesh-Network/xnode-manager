inputs:
{
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}:
{
  disabledModules = [
    # This module adds values to multiple lists (systemPackages, supportedFilesystems)
    # which are impossible/unpractical to remove, so we disable the entire module.
    "profiles/base.nix"
  ];

  imports = [
    (modulesPath + "/installer/netboot/netboot-minimal.nix")
    # reduce closure size by removing perl
    "${modulesPath}/profiles/perlless.nix"
    # FIXME: we still are left with nixos-generate-config due to nixos-install-tools
    { system.forbiddenDependenciesRegexes = lib.mkForce [ ]; }
  ];

  boot.initrd.compressor = "xz";

  system.stateVersion = config.system.nixos.release;

  nix.settings = {
    extra-experimental-features = [
      "nix-command"
      "flakes"
    ];
    accept-flake-config = true;
  };

  # https://github.com/nix-community/nixos-images/blob/main/nix/kexec-installer/module.nix#L50
  system.build.kexecInstallerTarball = pkgs.runCommand "kexec-tarball" { } ''
    mkdir xnodeos $out
    cp "${config.system.build.netbootRamdisk}/initrd" xnodeos/initrd
    cp "${config.system.build.kernel}/${config.system.boot.loader.kernelFile}" xnodeos/bzImage
    cp "${config.system.build.kexecScript}" xnodeos/install
    cp "${pkgs.pkgsStatic.kexec-tools}/bin/kexec" xnodeos/kexec
    tar -czvf $out/OSkexec-${pkgs.stdenv.hostPlatform.system}.tar.gz xnodeos
  '';

  # https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/installer/netboot/netboot.nix#L120
  # Modify kexec-boot to pass env variables to kexec environment
  system.build.kexecScript = lib.mkForce (
    pkgs.writeScript "kexec-boot" ''
      #!/usr/bin/env bash
      SCRIPT_DIR=$( cd -- "$( dirname -- "''${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
      ''${SCRIPT_DIR}/kexec --load ''${SCRIPT_DIR}/bzImage \
        --initrd=''${SCRIPT_DIR}/initrd \
        --command-line "init=${config.system.build.toplevel}/init ${toString config.boot.kernelParams} && $(cat << EOF

      export XNODE_OWNER="''${XNODE_OWNER}" && export DOMAIN="''${DOMAIN}" && export ACME_EMAIL="''${ACME_EMAIL}" && export USER_PASSWD="''${USER_PASSWD}" && export ENCRYPTED="''${ENCRYPTED}"
      EOF
      )"
      ''${SCRIPT_DIR}/kexec -e
    ''
  );

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

  # Reduce closure size (https://github.com/nix-community/nixos-images/blob/main/nix/noninteractive.nix)
  documentation.enable = false;
  documentation.man.man-db.enable = false;
  system.installer.channel.enable = false;

  # nixos-option is mainly useful for interactive installations
  system.tools.nixos-option.enable = false;

  # among others, this prevents carrying a stdenv with gcc in the image
  system.extraDependencies = lib.mkForce [ ];

  # prevents shipping nixpkgs, unnecessary if system is evaluated externally
  nix.registry = lib.mkForce { };

  # would pull in nano
  programs.nano.enable = false;

  # prevents strace
  environment.defaultPackages = lib.mkForce [
    pkgs.parted
    pkgs.gptfdisk
    pkgs.e2fsprogs
  ];

  # included in systemd anyway
  systemd.sysusers.enable = true;
  services.userborn.enable = false;

  # normal users are not allowed with sys-users
  # see https://github.com/NixOS/nixpkgs/pull/328926
  users.users.nixos = {
    isSystemUser = true;
    isNormalUser = lib.mkForce false;
    shell = "/run/current-system/sw/bin/bash";
    group = "nixos";
  };
  users.groups.nixos = { };

  # we have still run0 from systemd and most of the time we just use root
  security.sudo.enable = false;
  security.polkit.enable = lib.mkForce false;

  documentation.man.enable = false;

  # no dependency on x11
  services.dbus.implementation = "broker";

  # introduces x11 dependencies
  security.pam.services.su.forwardXAuth = lib.mkForce false;

  # Don't install the /lib/ld-linux.so.2 stub. This saves one instance of nixpkgs.
  environment.ldso32 = null;

  # we prefer root as this is also what we use in nixos-anywhere
  services.getty.autologinUser = lib.mkForce "root";

  # we are missing this from base.nix
  boot.supportedFilesystems = [
    "ext4"
    "btrfs"
    ## quiet huge dependency closure
    #"cifs"
    "f2fs"
    ## anyone still using this over ext4?
    #"jfs"
    "ntfs"
    ## no longer seems to be maintained, anyone still using it?
    #"reiserfs"
    "vfat"
    "xfs"
  ];
  boot.kernelModules = [
    # we have to explicitly enable this, otherwise it is not loaded even when creating a raid:
    # https://github.com/nix-community/nixos-anywhere/issues/249
    "dm-raid"
  ];
}
