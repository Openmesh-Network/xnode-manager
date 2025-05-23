{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-reverse-proxy;
in
{
  options = {
    services.xnode-reverse-proxy = {
      enable = lib.mkEnableOption "Enable Xnode Reverse Proxy.";

      program = {
        type = lib.mkOption {
          type = lib.types.enum [
            "nginx"
            "cloudflared"
          ];
          default = "nginx";
          example = "cloudflared";
          description = ''
            Reverse proxy program to use.
          '';
        };

        cloudflared = {
          tunnel = {
            name = lib.mkOption {
              type = lib.types.str;
              default = "xnode";
              example = "MyXnode";
              description = ''
                Name of the tunnel to create and connect to in Cloudflare.
              '';
            };
          };
        };
      };

      rules = lib.mkOption {
        type = lib.types.attrsOf (
          lib.types.listOf (
            lib.types.submodule {
              options = {
                forward = lib.mkOption {
                  type = lib.types.str;
                  example = "http://xnode.local:3000";
                  description = ''
                    Where to forward the request to.
                  '';
                };

                path = lib.mkOption {
                  type = lib.types.nullOr lib.types.str;
                  default = null;
                  example = "/page";
                  description = ''
                    Path of the incoming request .
                  '';
                };
              };
            }
          )
        );
        default = { };
        example = {
          "example.com" = [
            {
              path = "/page1";
              forward = "http://localhost:3001";
            }
            { forward = "http://localhost:3000"; }
          ];
          "test.example.com" = [
            { forward = "https://container:80"; }
          ];
        };
        description = ''
          Rules to configure the reverse proxy forwarding.
        '';
      };

      openFirewall = lib.mkOption {
        type = lib.types.bool;
        default = true;
        example = false;
        description = ''
          Open required firewall ports for the reverse proxy to function.
        '';
      };
    };
  };

  config =
    let
      data = "/var/lib/xnode-reverse-proxy";
    in
    lib.mkIf cfg.enable {
      users.groups.xnode-reverse-proxy = { };
      users.users.xnode-reverse-proxy = {
        isSystemUser = true;
        group = "xnode-reverse-proxy";
        home = data;
        createHome = true;
      };

      networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall (
        if (cfg.program.type == "nginx") then
          [
            80
            443
          ]
        else if (cfg.program.type == "cloudflared") then
          [ ]
        else
          [ ]
      );

      services.nginx = lib.mkIf (cfg.program.type == "nginx") {
        enable = true;
        user = "xnode-reverse-proxy";
        group = "xnode-reverse-proxy";

        recommendedOptimisation = true;
        recommendedProxySettings = true;
        recommendedTlsSettings = true;

        virtualHosts = lib.attrsets.mapAttrs (domain: rule: {
          enableACME = true;
          forceSSL = true;
          locations = builtins.listToAttrs (
            builtins.map (
              location:
              (lib.attrsets.nameValuePair (if (location.path == null) then "/" else location.path) {
                proxyPass = location.forward;
              })
            ) rule
          );
        }) cfg.rules;
      };

      systemd.services.cloudflared-login = lib.mkIf (cfg.program.type == "cloudflared") {
        wantedBy = [ "multi-user.target" ];
        description = "Authenticate cloudflared with your account.";
        after = [ "network.target" ];
        serviceConfig = {
          User = "xnode-reverse-proxy";
          Group = "xnode-reverse-proxy";
          Restart = "on-failure";
        };
        script = ''
          ${lib.getExe pkgs.cloudflared} tunnel login
        '';
      };

      systemd.paths.cloudflared-tunnel-xnode-create = lib.mkIf (cfg.program.type == "cloudflared") {
        wantedBy = [ "multi-user.target" ];
        pathConfig = {
          PathChanged = "${data}/.cloudflared/cert.pem";
          Unit = "cloudflared-tunnel-xnode-create.service";
        };
      };
      systemd.services.cloudflared-tunnel-xnode-create = lib.mkIf (cfg.program.type == "cloudflared") {
        description = "Create locally managed xnode tunnel.";
        serviceConfig = {
          User = "xnode-reverse-proxy";
          Group = "xnode-reverse-proxy";
          Restart = "on-failure";
        };
        script = ''
          ${lib.getExe pkgs.cloudflared} tunnel create "${cfg.program.cloudflared.tunnel.name}"
          for f in ${data}/.cloudflared/*.json ; do mv "$f" "${data}/.cloudflared/tunnel.json"; done
        '';
      };

      systemd.paths.cloudflared-tunnel-xnode = lib.mkIf (cfg.program.type == "cloudflared") {
        wantedBy = [ "multi-user.target" ];
        pathConfig = {
          PathExists = "${data}/.cloudflared/tunnel.json";
          Unit = "cloudflared-tunnel-xnode.service";
        };
      };
      systemd.services.cloudflared-tunnel-xnode.wantedBy = lib.mkIf (cfg.program.type == "cloudflared") (
        lib.mkForce [ ]
      );
      services.cloudflared = lib.mkIf (cfg.program.type == "cloudflared") {
        enable = true;
        user = "xnode-reverse-proxy";
        group = "xnode-reverse-proxy";

        tunnels."xnode" = {
          credentialsFile = "${data}/.cloudflared/tunnel.json";
          default = "http_status:404";
          ingress = lib.attrsets.foldlAttrs (
            acc: name: value:
            (lib.mkMerge [
              acc
              (builtins.listToAttrs (
                lib.lists.imap0 (
                  i: value:
                  lib.attrsets.nameValuePair name {
                    # hostname = name;
                    service = value.forward;
                    path = value.path;
                  }
                ) value
              ))
            ])
          ) { } cfg.rules;
        };
      };
    };
}
