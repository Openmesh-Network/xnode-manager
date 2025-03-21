#!/usr/bin/env bash

set -e # Stop on error
export HOME=/root # Cloud init might run without needed env variables

# Install Nix
sh <(curl -L https://nixos.org/nix/install) < /dev/null --daemon

# Enable Nix in current shell
. $HOME/.nix-profile/etc/profile.d/nix.sh

# Prepare output directory
mkdir -p /etc/nixos

# Perform nixos-facter hardware scan
nix run --option experimental-features "nix-command flakes" nixpkgs#nixos-facter -- -o /etc/nixos/facter.json

# Generate disk config
DISK_COUNTER=1
DISK_CONFIG_FILE="/etc/nixos/disk-config.nix"
ROOT_DISK=$(mount | grep "on / type" | awk '{print $1;}') # Find disk mounted on /
echo "{" > $DISK_CONFIG_FILE
echo "   fileSystems.\"/\" = { device = \"$ROOT_DISK\"; };" >> $DISK_CONFIG_FILE
for disk in $(fdisk -l | grep "Linux filesystem" | awk '{print $1;}'); do # Find all Linux filesystem partitions
   if [ "$disk" = "$ROOT_DISK" ]; then
      # Do not mount root disk on /mnt/disk
      continue
   fi

   echo "   fileSystems.\"$/mnt/disk$DISK_COUNTER\" = { device = \"$disk\"; };" >> $DISK_CONFIG_FILE
   DISK_COUNTER=$(expr $DISK_COUNTER + 1)
done
echo "}" >> $DISK_CONFIG_FILE

# Write XnodeOS configuration
(curl -L https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/flake.nix)> /etc/nixos/flake.nix
if [[ -v XNODE_OWNER ]]; then
   echo -n "${XNODE_OWNER}" > /etc/nixos/xnode-owner
fi
if [[ -v USER_PASSWD ]]; then
   echo -n "${USER_PASSWD}" > /etc/nixos/user-passwd
fi

BOOT_POINT="/boot"
# Set common efi partition mounting points
if mountpoint -q /efi; then
   BOOT_POINT="/efi"
fi
if mountpoint -q /boot/efi; then
   BOOT_POINT="/boot/efi"
fi
echo -n "${BOOT_POINT}" > /etc/nixos/bootpoint

# Build configuration
nix build --option experimental-features "nix-command flakes" /etc/nixos#nixosConfigurations.xnode.config.system.build.toplevel --accept-flake-config

# Apply configuration
nix-env -p /nix/var/nix/profiles/system --set ./result

# Switch OS to Nix
touch /etc/NIXOS && touch /etc/NIXOS_LUSTRATE && echo /etc/nixos | tee -a /etc/NIXOS_LUSTRATE
NIXOS_INSTALL_BOOTLOADER=1 /nix/var/nix/profiles/system/bin/switch-to-configuration boot

# Boot into new OS
reboot