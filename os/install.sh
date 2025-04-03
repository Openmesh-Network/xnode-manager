#!/usr/bin/env bash

set -e # Stop on error

# Ignore kernel update popup blocking apt install
sed -i "s/#\$nrconf{kernelhints} = -1;/\$nrconf{kernelhints} = -1;/g" /etc/needrestart/needrestart.conf

# Install kexec on Ubuntu
apt install kexec-tools

curl -L https://github.com/Openmesh-Network/xnode-manager/releases/download/OSkexec/OSkexec.tar.gz | tar -xzf- -C /root

# Boot into kexec
/root/kexec-boot