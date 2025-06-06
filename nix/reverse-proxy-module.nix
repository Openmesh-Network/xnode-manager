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
                    Path of the incoming request.
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
      rules = builtins.mapAttrs (
        domain: rule:
        builtins.foldl'
          (
            acc: entry:
            let
              forward_split = lib.splitString "://" entry.forward;
              protocol = builtins.elemAt forward_split 0;
              server_split = lib.splitString ":" (builtins.elemAt forward_split 1);
              server = builtins.elemAt server_split 0;
              port = builtins.elemAt server_split 1;
              parsedEntry = {
                protocol = protocol;
                server = server;
                port = port;
              };
              http = protocol == "http" || protocol == "https";
              path = if (entry.path == null) then "/" else entry.path;
            in
            {
              http =
                acc.http // (if http then { ${path} = (acc.http.${path} or [ ]) ++ [ parsedEntry ]; } else { });
              stream = acc.stream ++ (if http then [ ] else [ parsedEntry ]);
            }
          )
          {
            http = { };
            stream = [ ];
          }
          rule
      ) cfg.rules;
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
                  ++ (builtins.map (entry: lib.toInt entry.port) (
                    builtins.filter (entry: entry.protocol == "tcp") rule.stream
                  ))
                )
              ) [ ] rules);
            allowedUDPPorts = (
              lib.attrsets.foldlAttrs (
                acc: name: rule:
                (
                  acc
                  ++ (builtins.map (entry: lib.toInt entry.port) (
                    builtins.filter (entry: entry.protocol == "udp") rule.stream
                  ))
                )
              ) [ ] rules
            );
          }
        else if (cfg.program.type == "cloudflared") then
          { }
        else
          { }
      );

      services.nginx = {
        enable = true;
        user = "xnode-reverse-proxy";
        group = "xnode-reverse-proxy";

        recommendedOptimisation = true;
        recommendedProxySettings = true;
        recommendedTlsSettings = true;

        upstreams = lib.attrsets.foldlAttrs (
          upstreamAcc: domain: rule:
          lib.mkMerge [
            upstreamAcc
            (lib.attrsets.foldlAttrs (
              domainAcc: path: entries:
              lib.mkMerge [
                domainAcc
                {
                  # Forward slash characters cannot be escaped inside proxy pass
                  "${domain}_${builtins.replaceStrings [ "/" ] [ "<slash>" ] path}".servers = lib.mkMerge (
                    builtins.map (entry: {
                      "${entry.server}:${entry.port}" = { };
                    }) entries
                  );
                }
              ]
            ) { } rule.http)
          ]
        ) { } rules;

        virtualHosts = builtins.mapAttrs (
          domain: rule:
          lib.mkIf ((builtins.length (builtins.attrNames rule.http)) > 0) (
            lib.mkMerge [
              (lib.mkIf (cfg.program.type == "nginx") {
                # NGINX is always used internally, only enable SSL in case it's the exposed reverse proxy service
                enableACME = true;
                forceSSL = true;
              })
              {
                locations = builtins.mapAttrs (path: entries: {
                  proxyWebsockets = true;
                  proxyPass = "${(builtins.elemAt entries 0).protocol}://${domain}_${
                    builtins.replaceStrings [ "/" ] [ "<slash>" ] path
                  }"; # NGINX doesn't allow upstreams with different protocols
                }) rule.http;
              }
            ]
          )
        ) rules;

        streamConfig = lib.attrsets.foldlAttrs (
          streamAcc: domain: rule:
          (lib.mkMerge [
            streamAcc
            (
              let
                upstreams = builtins.foldl' (
                  acc: entry:
                  acc
                  // {
                    "${domain}_${entry.protocol}_${entry.port}" = {
                      listen = "${entry.port}${if entry.protocol == "udp" then " udp" else ""}";
                      servers = (acc."${domain}_${entry.protocol}_${entry.port}".servers or [ ]) ++ [
                        "server ${entry.server}:${entry.port};"
                      ];
                    };
                  }
                ) { } rule.stream;
              in
              lib.attrsets.foldlAttrs (
                serverAcc: upstream_name: upstream_value:
                lib.mkMerge [
                  serverAcc
                  ''
                    upstream ${upstream_name} {
                      ${builtins.concatStringsSep "\n" upstream_value.servers}
                    }

                    server {
                      server_name ${domain};
                      listen ${upstream_value.listen};
                      proxy_pass ${upstream_name};
                    }
                  ''
                ]
              ) '''' upstreams
            )
          ])
        ) '''' rules;
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
      systemd.services.cloudflared-tunnel-xnode = lib.mkIf (cfg.program.type == "cloudflared") ({
        wantedBy = lib.mkForce [ ];
        serviceConfig.User = lib.mkForce "xnode-reverse-proxy";
        serviceConfig.Group = lib.mkForce "xnode-reverse-proxy";
        serviceConfig.DynamicUser = lib.mkForce false;
      });
      services.cloudflared = lib.mkIf (cfg.program.type == "cloudflared") {
        enable = true;
        tunnels."xnode" = {
          credentialsFile = "${data}/.cloudflared/tunnel.json";
          default = "http://localhost"; # Query NGINX http
          ingress = lib.attrsets.foldlAttrs (
            acc: domain: rule:
            (lib.mkMerge [
              acc
              (lib.mkMerge (
                lib.lists.imap0 (i: entry: {
                  ${domain} = {
                    # hostname = name;
                    service = "${entry.protocol}://localhost:${entry.port}"; # Query NGINX stream
                  };
                }) rule.stream
              ))
            ])
          ) { } rules;

        };
      };
    };
}
