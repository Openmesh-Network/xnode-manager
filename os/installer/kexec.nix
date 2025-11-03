{
  inputs,
  modulesPath,
  config,
  pkgs,
  lib,
  ...
}@args:
{
  imports = [
    (modulesPath + "/installer/netboot/netboot.nix")
    (import ./config.nix args)
  ];

  boot.initrd.compressor = "xz";

  # https://github.com/nix-community/nixos-images/blob/main/nix/kexec-installer/module.nix#L50
  system.build.kexecInstallerTarball = pkgs.runCommand "kexec-tarball" { } ''
    mkdir xnodeos $out
    cp "${config.system.build.netbootRamdisk}/initrd" xnodeos/initrd
    cp "${config.system.build.kernel}/${config.system.boot.loader.kernelFile}" xnodeos/bzImage
    cp "${config.system.build.kexecScript}" xnodeos/install
    cp "${pkgs.pkgsStatic.kexec-tools}/bin/kexec" xnodeos/kexec
    cp "${pkgs.pkgsStatic.iproute2.override { iptables = null; }}/bin/ip" xnodeos/ip
    tar -czvf $out/xnodeos-kexec-installer-${pkgs.stdenv.hostPlatform.system}.tar.gz xnodeos
  '';

  # https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/installer/netboot/netboot.nix#L120
  # Modify kexec-boot to pass env variables to kexec environment
  system.build.kexecScript = lib.mkForce (
    pkgs.writeScript "kexec-boot" ''
      #!/usr/bin/env bash
      SCRIPT_DIR=$( cd -- "$( dirname -- "''${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
      NETWORK_CONFIG="{ \"address\": $(''${SCRIPT_DIR}/ip -j address show), \"route\":  $(''${SCRIPT_DIR}/ip -j route show) }"
      ''${SCRIPT_DIR}/kexec --load ''${SCRIPT_DIR}/bzImage \
        --initrd=''${SCRIPT_DIR}/initrd \
        --command-line "init=${config.system.build.toplevel}/init ${toString config.boot.kernelParams} && $(cat << EOF

      export XNODE_OWNER="''${XNODE_OWNER}" && export DOMAIN="''${DOMAIN}" && export ACME_EMAIL="''${ACME_EMAIL}" && export USER_PASSWD="''${USER_PASSWD}" && export ENCRYPTED="''${ENCRYPTED}" && export NETWORK_CONFIG="''${NETWORK_CONFIG}" && export INITIAL_CONFIG="''${INITIAL_CONFIG}"
      EOF
      )"
      ''${SCRIPT_DIR}/kexec -e
    ''
  );

  systemd.services.install-xnodeos.script = lib.mkBefore ''
    # Extract environmental variables
    sed '2q;d' /proc/cmdline > /tmp/xnode-env
    source /tmp/xnode-env
  '';

  systemd.services.apply-network-config = {
    wantedBy = [ "multi-user.target" ];
    description = "Apply run time provided network config.";
    wants = [ "network-pre.target" ];
    before = [ "network-pre.target" ];
    serviceConfig = {
      Type = "oneshot";
      User = "root";
      Group = "root";
      RemainAfterExit = true;
    };
    path = [
      pkgs.iproute2
      pkgs.jq
    ];
    script = ''
      # Extract environmental variables
      sed '2q;d' /proc/cmdline > /tmp/xnode-env
      source /tmp/xnode-env

      if [[ $NETWORK_CONFIG ]]; then
        interfaces=$(echo "$NETWORK_CONFIG" | jq -c '.address.[]')
        routes=$(echo "$NETWORK_CONFIG" | jq -c '.route.[]')
        for iface in $interfaces; do
          mac=$(echo "$iface" | jq -r '.address')
          og_name=$(echo "$iface" | jq -r '.ifname')
          name=$(grep -l "$mac" /sys/class/net/*/address | sed 's|/sys/class/net/\(.*\)/address|\1|')

          addresses=$(echo "$iface" | jq -c '.addr_info[]')
          for address in $addresses; do
            scope=$(echo "$address" | jq -r '.scope')
            dynamic=$(echo "$address" | jq -r '.dynamic')

            if [ "$scope" != "global" ] || [ "$dynamic" = "true" ]; then
                continue
            fi

            config="$(echo "$address" | jq -r '.local')/$(echo "$address" | jq -r '.prefixlen')"
            ip address add $config dev $name
          done

          for route in $routes; do
            protocol=$(echo "$route" | jq -r '.protocol')
            dev=$(echo "$route" | jq -r '.dev')

            if [ "$protocol" != "static" ] || [ "$dev" != "$og_name" ]; then
                continue
            fi

            config="$(echo "$route" | jq -r '.dst') via $(echo "$route" | jq -r '.gateway')"
            ip route add $config dev $name
          done
        done
      fi
    '';
  };
}
