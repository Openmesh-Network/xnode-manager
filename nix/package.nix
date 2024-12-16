{ rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "xnode-rust-template";
  version = "1.0.0";
  src = ../rust-app;

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  meta = {
    mainProgram = "xnode-rust-template";
  };
}
