{
  rustPlatform,
  pkg-config,
  openssl,
}:
rustPlatform.buildRustPackage {
  pname = "xnode-manager";
  version = "1.0.0";
  src = ../rust-app;

  nativeBuildInputs = [
    pkg-config
    openssl.dev
  ];

  cargoLock = {
    lockFile = ../rust-app/Cargo.lock;
  };

  doDist = false;

  meta = {
    mainProgram = "xnode-manager";
  };
}
