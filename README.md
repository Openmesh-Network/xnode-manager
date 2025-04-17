## Xnode Manager

Allow configuring and monitoring your Xnode through external platforms, such as Xnode Studio.

## Commands (in root folder)

```
nix run
```

## Commands (in rust-app)

```
cargo build
cargo run
```

## XnodeOS Install

> [!CAUTION]
> THIS WILL OVERWRITE THE CURRENTLY INSTALLED OS AND ALL ITS DATA, INCLUDING ANY ATTACHED DRIVES!

NixOS installation with custom XnodeOS configuration replacing an existing OS installation (e.g Ubuntu 24.04). Performs steps based on https://nixos.org/manual/nixos/stable/index.html#sec-installing-from-other-distro. This command should be run as root.

XNODE_OWNER env var should be set when deploying in a open-port environment to prevent malicious actors from claiming your Xnode before you.
DOMAIN and ACME_EMAIL en vars should be set to communicate with xnode-manager over HTTPS. Before accessing any confidential information on your Xnode, you are recommended to enable HTTPS. DOMAIN (can be a subdomain) should have an A record point to this Xnode. ACME_EMAIL can not be a blacklisted email (e.g. @example.com).

USER_PASSWD env var can be set to allow password login as user "xnode". However it is recommended to manage your machine through this manager app only.

ENCRYPTED env var is recommended to be set, it will fully encrypt all drives (with unattended TPM2 decryption on boot) and enable Secure Boot. This protects against malicious actors with physical access to your Xnode.

```
export ENCRYPTED="1" && curl https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/install.sh | bash 2>&1 | tee /tmp/xnodeos.log
```

### Cloud Init

```
#cloud-config
runcmd:
 - export DOMAIN="xnode.plopmenz.com" && export ACME_EMAIL="plopmenz@gmail.com" && export XNODE_OWNER="eth:519ce4C129a981B2CBB4C3990B1391dA24E8EbF3" && export ENCRYPTED="1" && curl https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/install.sh | bash 2>&1 | tee /tmp/xnodeos.log
```
