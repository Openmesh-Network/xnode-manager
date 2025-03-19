#!/usr/bin/env bash

set -e # Stop on error
export HOME=/root # Cloud init might run without needed env variables

# Install Nix
sh <(curl -L https://nixos.org/nix/install) < /dev/null --daemon

# Enable Nix in current shell
. $HOME/.nix-profile/etc/profile.d/nix.sh

# Install NixOS installation tools
nix-env -f '<nixpkgs>' -iA nixos-install-tools

# Generate initial configuration
# /etc/nixos is forced as directory (preventing rel2abs / to resolve to /root on certain systems)
`which nixos-generate-config` --dir /etc/nixos/

# Write XnodeOS configuration
(curl -L https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/flake.nix)> /etc/nixos/flake.nix
if [[ -v XNODE_OWNER ]]; then
   echo -n "${XNODE_OWNER}" > /etc/nixos/xnode-owner
fi
if [[ -v USER_PASSWD ]]; then
   echo -n "${USER_PASSWD}" > /etc/nixos/user-passwd
fi

# Build configuration
nix build /etc/nixos#nixosConfigurations.xnode.config.system.build.toplevel --extra-experimental-features nix-command --extra-experimental-features flakes --accept-flake-config

# Apply configuration
nix-env -p /nix/var/nix/profiles/system --set ./result

# Switch OS to Nix
touch /etc/NIXOS && touch /etc/NIXOS_LUSTRATE && echo etc/nixos | tee -a /etc/NIXOS_LUSTRATE
NIXOS_INSTALL_BOOTLOADER=1 /nix/var/nix/profiles/system/bin/switch-to-configuration boot

# Boot into new OS
reboot