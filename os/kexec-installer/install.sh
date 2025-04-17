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
      disk${DISK_COUNTER} = {
        device = "/dev/${disk}";
        type = "disk";
        content = {
          type = "gpt";
          partitions = {
EOL
  MOUNT_POINT="/mnt/disk${DISK_COUNTER}"
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
                  name = "disk${DISK_COUNTER}";
                  settings = {
                    allowDiscards = true;
                    bypassWorkqueues = true;
                  };
                  content = {
                    type = "filesystem";
                    format = "ext4";
                    mountpoint = "${MOUNT_POINT}";
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
                mountpoint = "${MOUNT_POINT}";
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
cat /tmp/secret.key | disko --mode destroy,format,mount /etc/nixos/disko-config.nix --yes-wipe-all-disks

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
  echo -n "${XNODE_OWNER}" > /mnt/etc/nixos/xnode-owner
fi
if [[ $DOMAIN ]]; then
  echo -n "${DOMAIN}" > /mnt/etc/nixos/domain
fi
if [[ $ACME_EMAIL ]]; then
  echo -n "${ACME_EMAIL}" > /mnt/etc/nixos/acme-email
fi
if [[ $USER_PASSWD ]]; then
  echo -n "${USER_PASSWD}" > /mnt/etc/nixos/user-passwd
fi

# Apply configuration
nixos-install --no-root-passwd --no-channel-copy --flake /mnt/etc/nixos#xnode

# Boot into new OS
reboot