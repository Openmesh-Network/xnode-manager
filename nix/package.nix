{ rustPlatform, pkgs }:
rustPlatform.buildRustPackage {
  pname = "xnode-manager";
  version = "1.0.1";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  buildInputs = with pkgs; [
    acl
  ];

  meta = {
    mainProgram = "xnode-manager";
  };
}
