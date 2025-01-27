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

## Fresh XnodeOS Install

NixOS installation with custom XnodeOS configuration. Performs steps based on https://nixos.org/manual/nixos/stable/index.html#sec-installing-from-other-distro. This command should be run as root. THIS WILL OVERWRITE THE CURRENTLY INSTALLED OS AND ALL ITS DATA!

```
sh <(curl -L https://raw.githubusercontent.com/Openmesh-Network/xnode-manager/main/os/install.sh)
```
