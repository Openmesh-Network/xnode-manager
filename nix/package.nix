{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "xnode-manager";
  version = "2.0.0";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  buildInputs = [
    pkgs.acl

    pkgs.pkg-config
    pkgs.systemdLibs
  ];

  meta = {
    mainProgram = "xnode-manager";
  };
}
