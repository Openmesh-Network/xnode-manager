{
  inputs,
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}@args:
{
  imports = [
    (modulesPath + "/installer/netboot/netboot-minimal.nix")
    (import ./config.nix args)
    ./minimal.nix
  ];

  boot.initrd.compressor = "xz";

  # https://github.com/nix-community/nixos-images/blob/main/nix/kexec-installer/module.nix#L50
  system.build.kexecInstallerTarball = pkgs.runCommand "kexec-tarball" { } ''
    mkdir xnodeos $out
    cp "${config.system.build.netbootRamdisk}/initrd" xnodeos/initrd
    cp "${config.system.build.kernel}/${config.system.boot.loader.kernelFile}" xnodeos/bzImage
    cp "${config.system.build.kexecScript}" xnodeos/install
    cp "${pkgs.pkgsStatic.kexec-tools}/bin/kexec" xnodeos/kexec
    tar -czvf $out/xnodeos-kexec-installer-${pkgs.stdenv.hostPlatform.system}.tar.gz xnodeos
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

  systemd.services.install-xnodeos.script = lib.mkBefore ''
    # Extract environmental variables
    sed '2q;d' /proc/cmdline > /tmp/xnode-env
    source /tmp/xnode-env
  '';
}
