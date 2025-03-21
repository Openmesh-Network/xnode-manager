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

USER_PASSWD env var can be set to allow password login as user "xnode". However it is recommended to manage your machine through this manager app only.

```
curl https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/install.sh | bash 2>&1 | tee /tmp/xnodeos.log
```

### Cloud Init

```
#cloud-config
runcmd:
 - export XNODE_OWNER=eth:519ce4C129a981B2CBB4C3990B1391dA24E8EbF3 && curl https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/install.sh | bash 2>&1 | tee /tmp/xnodeos.log
```
