{
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}:
{
  imports = [
    (modulesPath + "/installer/netboot/netboot-minimal.nix")
  ];

  boot.initrd.compressor = "xz";

  system.stateVersion = config.system.nixos.release;

  documentation.enable = false;
  documentation.man.man-db.enable = false;
  system.installer.channel.enable = false;

  nix.settings.extra-experimental-features = [
    "nix-command"
    "flakes"
  ];

  # https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/installer/netboot/netboot.nix#L120
  # Modify kexec-boot to pass env variables to kexec environment
  system.build.kexecScript = lib.mkForce (
    pkgs.writeScript "kexec-boot" ''
      #!/usr/bin/env bash
      if ! kexec -v >/dev/null 2>&1; then
        echo "kexec not found: please install kexec-tools" 2>&1
        exit 1
      fi
      SCRIPT_DIR=$( cd -- "$( dirname -- "''${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
      kexec --load ''${SCRIPT_DIR}/bzImage \
        --initrd=''${SCRIPT_DIR}/initrd.gz \
        --command-line "init=${config.system.build.toplevel}/init ${toString config.boot.kernelParams} && $(cat << EOF

      export XNODE_OWNER=''${XNODE_OWNER} && export DOMAIN=''${DOMAIN} && export ACME_EMAIL=''${ACME_EMAIL} && export USER_PASSWD=''${USER_PASSWD}
      EOF
      )"
      kexec -e
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
         if [ "$DISK_COUNTER" -eq 0 ]; then
            # Boot disk
            cat >> $DISK_CONFIG_FILE << EOL
            disk''${DISK_COUNTER} = {
              device = "/dev/''${disk}";
              type = "disk";
              content = {
                type = "gpt";
                partitions = {
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
                  root = {
                    size = "100%";
                    content = {
                      type = "filesystem";
                      format = "ext4";
                      mountpoint = "/";
                    };
                  };
                };
              };
            };
      EOL
         else
            # Data disk
            cat >> $DISK_CONFIG_FILE << EOL
            disk''${DISK_COUNTER} = {
              device = "/dev/''${disk}";
              type = "disk";
              content = {
                type = "gpt";
                partitions = {
                  root = {
                    size = "100%";
                    content = {
                      type = "filesystem";
                      format = "ext4";
                      mountpoint = "/mnt/disk''${DISK_COUNTER}";
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

      # Apply disk formatting and mount drives
      nix run github:nix-community/disko/latest -- --mode destroy,format,mount /etc/nixos/disko-config.nix --yes-wipe-all-disks

      # Move disko-config to root file system
      mkdir -p /mnt/etc/nixos
      mv /etc/nixos/disko-config.nix /mnt/etc/nixos

      # Perform nixos-facter hardware scan
      nixos-facter -o /mnt/etc/nixos/facter.json

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

      # Apply configuration
      nixos-install --no-root-passwd --no-channel-copy --system ./result

      # Boot into new OS
      reboot
    '';
  };
}
