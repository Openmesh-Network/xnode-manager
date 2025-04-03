#!/usr/bin/env bash

set -e # Stop on error

# Install kexec on Ubuntu
apt install kexec-tools

curl -L https://github.com/Openmesh-Network/xnode-manager/releases/download/OSkexec/OSkexec.tar.gz | tar -xzf- -C /root

# Boot into kexec
/root/kexec-boot