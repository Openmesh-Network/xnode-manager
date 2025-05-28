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
                  example = "http://xnode:3000";
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
          "play.example.com" = [
            { forward = "tcp://minecraft-server:25565"; }
            { forward = "udp://minecraft-server:25565"; }
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

      networking.firewall = lib.mkIf cfg.openFirewall (
        if (cfg.program.type == "nginx") then
          {
            allowedTCPPorts =
              [
                80
                443
              ]
              ++ (lib.attrsets.foldlAttrs (
                acc: name: rule:
                (
                  acc
                  ++ (builtins.map
                    (
                      entry:
                      let
                        forward_split = lib.splitString "://" entry.forward;
                        server = builtins.elemAt forward_split 1;
                        port = builtins.elemAt (lib.splitString ":" server) 1;
                      in
                      lib.toInt port
                    )
                    (
                      builtins.filter (
                        entry:
                        let
                          protocol = (builtins.elemAt (lib.splitString "://" entry.forward) 0);
                        in
                        protocol == "tcp"
                      ) rule
                    )
                  )
                )
              ) [ ] cfg.rules);
            allowedUDPPorts = lib.attrsets.foldlAttrs (
              acc: name: rule:
              (
                acc
                ++ (builtins.map
                  (
                    entry:
                    let
                      forward_split = lib.splitString "://" entry.forward;
                      server = builtins.elemAt forward_split 1;
                      port = builtins.elemAt (lib.splitString ":" server) 1;
                    in
                    lib.toInt port
                  )
                  (
                    builtins.filter (
                      entry:
                      let
                        protocol = (builtins.elemAt (lib.splitString "://" entry.forward) 0);
                      in
                      protocol == "udp"
                    ) rule
                  )
                )
              )
            ) [ ] cfg.rules;
          }
        else if (cfg.program.type == "cloudflared") then
          { }
        else
          { }
      );

      services.nginx = lib.mkIf (cfg.program.type == "nginx") {
        enable = true;
        user = "xnode-reverse-proxy";
        group = "xnode-reverse-proxy";

        recommendedOptimisation = true;
        recommendedProxySettings = true;
        recommendedTlsSettings = true;

        virtualHosts = lib.attrsets.mapAttrs (
          domain: rule:
          let
            httpEntries = (
              builtins.filter (
                entry:
                let
                  protocol = (builtins.elemAt (lib.splitString "://" entry.forward) 0);
                in
                protocol == "http" || protocol == "https"
              ) rule
            );
          in
          lib.mkIf ((builtins.length httpEntries) > 0) {
            enableACME = true;
            forceSSL = true;
            locations = builtins.listToAttrs (
              builtins.map (
                entry:
                (lib.attrsets.nameValuePair (if (entry.path == null) then "/" else entry.path) {
                  proxyWebsockets = true;
                  proxyPass = entry.forward;
                })
              ) httpEntries
            );
          }
        ) cfg.rules;

        streamConfig = lib.attrsets.foldlAttrs (
          acc: name: rule:
          (lib.mkMerge [
            acc
            (lib.mkMerge (
              builtins.map
                (
                  entry:
                  let
                    forward_split = lib.splitString "://" entry.forward;
                    protocol = builtins.elemAt forward_split 0;
                    server = builtins.elemAt forward_split 1;
                    port = builtins.elemAt (lib.splitString ":" server) 1;
                  in
                  ''
                    server {
                      server_name ${name};
                      listen ${port}${if protocol == "udp" then " udp" else ""};
                      proxy_pass ${server};
                    }
                  ''
                )
                (
                  builtins.filter (
                    entry:
                    let
                      protocol = (builtins.elemAt (lib.splitString "://" entry.forward) 0);
                    in
                    protocol == "tcp" || protocol == "udp"
                  ) rule
                )
            ))
          ])
        ) '''' cfg.rules;
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
            acc: name: rule:
            (lib.mkMerge [
              acc
              (builtins.listToAttrs (
                lib.lists.imap0 (
                  i: entry:
                  lib.attrsets.nameValuePair name {
                    # hostname = name;
                    service = entry.forward;
                    path = entry.path;
                  }
                ) rule
              ))
            ])
          ) { } cfg.rules;
        };
      };
    };
}
