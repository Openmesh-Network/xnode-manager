#!/usr/bin/env bash

set -e # Stop on error

# Download and extract kexec archive
curl -L "https://github.com/Openmesh-Network/xnode-manager/releases/download/installer/xnodeos-kexec-installer-$(uname -m)-linux.tar.gz" | tar -xzf- -C /root

# Boot into kexec
/root/xnodeos/install