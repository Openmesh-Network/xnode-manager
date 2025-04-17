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

    # Cache for disko
    substituters = [ "https://nix-community.cachix.org" ];
    trusted-public-keys = [ "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=" ];
  };

  # https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/installer/netboot/netboot.nix#L134
  # Modify kexec-tree to add kexec binary
  system.build.kexecTree = lib.mkForce (
    pkgs.linkFarm "kexec-tree" [
      {
        name = "initrd";
        path = "${config.system.build.netbootRamdisk}/initrd";
      }
      {
        name = "bzImage";
        path = "${config.system.build.kernel}/${config.system.boot.loader.kernelFile}";
      }
      {
        name = "kexec-boot";
        path = config.system.build.kexecScript;
      }
      {
        name = "kexec";
        path = "${pkgs.pkgsStatic.kexec-tools}/bin/kexec";
      }
    ]
  );

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
      pkgs.nixos-facter
      pkgs.sbctl
      pkgs.clevis
    ];
    script = ''
      # Extract environmental variables
      sed '2q;d' /proc/cmdline > /tmp/xnode-env
      source /tmp/xnode-env

      mkdir -p /etc/nixos

      # Generate disko-config.nix
      DISK_COUNTER=0
      DISK_CONFIG_FILE="/etc/nixos/disko-config.nix"
      cat > $DISK_CONFIG_FILE << EOL
      {
        disko.devices = {
          disk = {
      EOL
      for disk in $(lsblk  --nodeps  --json | jq '.blockdevices[] | select(.type == "disk" and .rm == false and .ro == false) | .name' -r); do # Find all attached disks
        cat >> $DISK_CONFIG_FILE << EOL
            disk''${DISK_COUNTER} = {
              device = "/dev/''${disk}";
              type = "disk";
              content = {
                type = "gpt";
                partitions = {
      EOL
        MOUNT_POINT="/mnt/disk''${DISK_COUNTER}"
        if [ "$DISK_COUNTER" -eq 0 ]; then
          # Boot disk
          MOUNT_POINT="/"
          cat >> $DISK_CONFIG_FILE << EOL
                  boot = {
                    size = "1M";
                    type = "EF02"; # for grub MBR
                  };
                  ESP = {
                    type = "EF00";
                    size = "1G";
                    content = {
                      type = "filesystem";
                      format = "vfat";
                      mountpoint = "/boot";
                      mountOptions = [ "umask=0077" ];
                    };
                  };
      EOL
        fi
        if [[ $ENCRYPTED ]]; then
          cat >> $DISK_CONFIG_FILE << EOL
                    luks = {
                      size = "100%";
                      content = {
                        type = "luks";
                        name = "disk''${DISK_COUNTER}";
                        settings = {
                          allowDiscards = true;
                          bypassWorkqueues = true;
                        };
                        content = {
                          type = "filesystem";
                          format = "ext4";
                          mountpoint = "''${MOUNT_POINT}";
                        };
                      };
                    };
                  };
                };
              };
      EOL
        else
          cat >> $DISK_CONFIG_FILE << EOL
                  root = {
                    size = "100%";
                    content = {
                      type = "filesystem";
                      format = "ext4";
                      mountpoint = "''${MOUNT_POINT}";
                    };
                  };
                };
              };
            };
      EOL
        fi
         DISK_COUNTER=$(expr $DISK_COUNTER + 1)
      done
      cat >> $DISK_CONFIG_FILE << EOL
          };
        };
      }
      EOL

      # Generate disk encryption key
      echo -n "$(tr -dc '[:alnum:]' < /dev/random | head -c64)" > /tmp/secret.key

      # Apply disk formatting and mount drives
      cat /tmp/secret.key | nix run github:nix-community/disko/latest -- --mode destroy,format,mount /etc/nixos/disko-config.nix --yes-wipe-all-disks

      # Move disko-config to root file system
      mkdir -p /mnt/etc/nixos
      mv /etc/nixos/disko-config.nix /mnt/etc/nixos

      # Perform nixos-facter hardware scan
      nixos-facter -o /mnt/etc/nixos/facter.json

      if [[ $ENCRYPTED ]]; then
        # Generate Secure Boot Keys
        mkdir -p /mnt/etc/secureboot
        sbctl create-keys --export /mnt/etc/secureboot/keys --database-path /mnt/etc/secureboot

        # Encrypt disk password for unattended (TPM2) boot decryption (Clevis)
        # Initially do not bind to any pcrs (always allow decryption) for the first boot
        # Set pcrs after first boot (to capture the TPM2 register values of XnodeOS instead of XnodeOS installer)
        cat /tmp/secret.key | clevis encrypt tpm2 '{"pcr_ids": ""}' > /mnt/etc/nixos/clevis.jwe

        # Mark system as encrypted
        echo -n "1" > /mnt/etc/nixos/encrypted
      fi

      # Set main configuration
      (curl -L https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/flake.nix)> /mnt/etc/nixos/flake.nix
      if [[ $XNODE_OWNER ]]; then
        echo -n "''${XNODE_OWNER}" > /mnt/etc/nixos/xnode-owner
      fi
      if [[ $DOMAIN ]]; then
        echo -n "''${DOMAIN}" > /mnt/etc/nixos/domain
      fi
      if [[ $ACME_EMAIL ]]; then
        echo -n "''${ACME_EMAIL}" > /mnt/etc/nixos/acme-email
      fi
      if [[ $USER_PASSWD ]]; then
        echo -n "''${USER_PASSWD}" > /mnt/etc/nixos/user-passwd
      fi

      # Build configuration
      nix build /mnt/etc/nixos#nixosConfigurations.xnode.config.system.build.toplevel --accept-flake-config

      # Copy over Nix store
      mkdir -p /mnt/nix/store
      cp /nix/store/* /mnt/nix/store -r -u

      # Apply configuration
      nixos-install --no-root-passwd --no-channel-copy --system ./result

      # Boot into new OS
      reboot
    '';
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
