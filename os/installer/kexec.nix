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
    cp "${config.system.build.netbootRamdisk}/initrd" xnodeos/initrd-base
    cp "${config.system.build.kernel}/${config.system.boot.loader.kernelFile}" xnodeos/bzImage
    cp "${config.system.build.kexecScript}" xnodeos/install
    cp "${pkgs.pkgsStatic.kexec-tools}/bin/kexec" xnodeos/kexec
    cp "${pkgs.pkgsStatic.coreutils}/bin/cp" xnodeos/cp
    cp "${pkgs.pkgsStatic.coreutils}/bin/mkdir" xnodeos/mkdir
    cp "${pkgs.pkgsStatic.findutils}/bin/find" xnodeos/find
    cp "${pkgs.pkgsStatic.iproute2.override { iptables = null; }}/bin/ip" xnodeos/ip
    cp "${pkgs.pkgsStatic.cpio}/bin/cpio" xnodeos/cpio
    tar -czvf $out/xnodeos-kexec-installer-${pkgs.stdenv.hostPlatform.system}.tar.gz xnodeos
  '';

  # https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/installer/netboot/netboot.nix#L120
  # Modify kexec-boot to pass env variables to kexec environment
  system.build.kexecScript = lib.mkForce (
    pkgs.writeScript "kexec-boot" ''
      #!/usr/bin/env bash
      SCRIPT_DIR=$( cd -- "$( dirname -- "''${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
      cd ''${SCRIPT_DIR}

      ./mkdir -p ./xnode-config

      NETWORK_CONFIG=$(echo "{ \"address\": $(./ip --json address show), \"route\":  $(./ip --json route show) }" | sed 's/"/\\"/g')
      cat << EOF > ./xnode-config/env
      export XNODE_OWNER="''${XNODE_OWNER}" && export DOMAIN="''${DOMAIN}" && export ACME_EMAIL="''${ACME_EMAIL}" && export USER_PASSWD="''${USER_PASSWD}" && export ENCRYPTED="''${ENCRYPTED}" && export NETWORK_CONFIG="''${NETWORK_CONFIG}" && export INITIAL_CONFIG="''${INITIAL_CONFIG}"
      EOF

      cp ./initrd-base ./initrd
      ./find ./xnode-config | ./cpio --format newc --create >> ./initrd

      ./kexec --load ./bzImage \
        --initrd=./initrd \
        --command-line "init=${config.system.build.toplevel}/init ${toString config.boot.kernelParams}"
      ./kexec -e
    ''
  );

  # https://github.com/nix-community/nixos-images/blob/main/nix/restore-remote-access.nix
  boot.initrd.systemd.enable = true;
  boot.initrd.systemd.services.restore-config-from-initrd = {
    unitConfig = {
      DefaultDependencies = false;
      RequiresMountsFor = "/sysroot /dev";
    };
    wantedBy = [ "initrd.target" ];
    requiredBy = [ "rw-etc.service" ];
    before = [ "rw-etc.service" ];
    serviceConfig.Type = "oneshot";
    script = ''
      cp -r xnode-config /sysroot
    '';
  };

  systemd.services.install-xnodeos.script = lib.mkBefore ''
    # Extract environmental variables
    source /xnode-config/env
  '';

  # https://github.com/nix-community/nixos-images/blob/main/nix/kexec-installer/restore_routes.py
  networking.firewall.enable = false;
  networking.useNetworkd = true;
  systemd.network.enable = true;
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
      source /xnode-config/env

      output="/etc/systemd/network"
      if [[ $NETWORK_CONFIG ]]; then
        interfaces=$(echo "$NETWORK_CONFIG" | jq -c '.address.[]')
        routes=$(echo "$NETWORK_CONFIG" | jq -c '.route.[]')
        for iface in $interfaces; do
          mac=$(echo "$iface" | jq -r '.address')
          name=$(echo "$iface" | jq -r '.ifname')
          systemd="''${output}/00-''${mac}.network"

          cat << EOF > "$systemd"
      [Match]
      MACAddress = $mac

      [Network]
      DHCP = yes
      LLDP = yes
      IPv6AcceptRA = yes
      MulticastDNS = yes
      EOF

          addresses=$(echo "$iface" | jq -c '.addr_info[]')
          for address in $addresses; do
            scope=$(echo "$address" | jq -r '.scope')
            dynamic=$(echo "$address" | jq -r '.dynamic')

            if [ "$scope" != "global" ] || [ "$dynamic" = "true" ]; then
                continue
            fi

            ip="$(echo $address | jq -r '.local')/$(echo $address | jq -r '.prefixlen')"

            cat << EOF >> "$systemd"
      Address = $ip
      EOF
          done

          for route in $routes; do
            protocol=$(echo "$route" | jq -r '.protocol')
            dev=$(echo "$route" | jq -r '.dev')

            if [ "$protocol" != "static" ] || [ "$dev" != "$name" ]; then
                continue
            fi

            onlink="no"
            flags=$(echo "$route" | jq -r '.flags')
            if [[ $flags == *"onlink"* ]]; then
              onlink="yes"
            fi

            destination=$(echo $route | jq -r '.dst')
            if [ "$destination" == "default" ]; then
              destination="0.0.0.0/0"
            fi
            gateway=$(echo $route | jq -r '.gateway')

            cat << EOF >> "$systemd"

      [Route]
      Destination = $destination
      Gateway = $gateway
      GatewayOnLink = $onlink
      EOF
          done
        done
      fi
    '';
  };
}
