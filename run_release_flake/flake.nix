{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/22.05";
    utils.url = "github:numtide/flake-utils";
    utils.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        lib = pkgs.lib;
      in {
        defaultPackage = pkgs.stdenv.mkDerivation rec {
          name = "aptakube";
          version = "1.2.1";

          src = pkgs.fetchurl {
            url = "https://github.com/TyberiusPrime/florg/releases/download/app-v0.0.0/florg_0.0.0_amd64.deb";
            sha256 = "sha256-00foHJey+sRWIXPBth2ASNSIC4nKsnS8BSrN232OBS8=";
          };

          buildInputs = [pkgs.dpkg pkgs.tree];

          sourceRoot = ".";
          unpackCmd = "dpkg-deb -x $src .";

          dontConfigure = true;
          dontBuild = true;

          installPhase = ''
            mkdir -p $out/bin
            cp  usr/bin/* $out/bin
          '';
          preFixup = let
              # found all these libs with `patchelf --print-required` and added them so that they are dynamically linked
            libPath = lib.makeLibraryPath [
              pkgs.webkitgtk
              pkgs.gtk3
              pkgs.pango
              pkgs.cairo
              pkgs.gdk-pixbuf
              pkgs.glib
              pkgs.openssl_3_0
              pkgs.notmuch
              pkgs.dbus
            ];
          in ''
            patchelf \
              --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" \
              --set-rpath "${libPath}" \
              $out/bin/*
          '';

          meta = with lib; {
            homepage = https://aptakube.com/;
            description = "Florg, because my brain is to janky for janky orgmode";
            license = licenses.mit;
            platforms = platforms.linux;
          };
        };
      }
    );
}
