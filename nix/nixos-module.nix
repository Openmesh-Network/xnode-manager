{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-rust-template;
  xnode-rust-template = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.xnode-rust-template = {
      enable = lib.mkEnableOption "Enable the rust app";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.xnode-rust-template = {
      wantedBy = [ "multi-user.target" ];
      description = "Rust App.";
      after = [ "network.target" ];
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-rust-template}";
        DynamicUser = true;
        CacheDirectory = "rust-app";
      };
    };
  };
}
