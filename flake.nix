{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?tag=22.10"; # that's 21.05
    utils.url = "github:numtide/flake-utils";
    utils.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    #mozillapkgs = {
    #url = "github:mozilla/nixpkgs-mozilla";
    #flake = false;
    #};
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      #pkgs = nixpkgs.legacyPackages."${system}";
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      rust = pkgs.rust-bin.stable."1.67.1".default.override {
        targets = ["x86_64-unknown-linux-musl"];
      };

      # Override the version used in naersk
      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      libraries = with pkgs; [
        webkitgtk
        gtk3
        cairo
        gdk-pixbuf
        glib
        dbus
        openssl_3
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-good
        notmuch
        xorg.libXtst # for rdev
      ];

      da_packages = with pkgs; [
        curl
        wget
        pkg-config
        dbus
        openssl_3
        glib
        gtk3
        libsoup
        webkitgtk
        notmuch
        xorg.libXtst # for rdev
      ];
    in rec {
      # `nix build`
      packages.my-project = naersk-lib.buildPackage {
        pname = "florg";
        root = ./.;
      };
      defaultPackage = packages.my-project;

      # `nix run`
      apps.my-project = utils.lib.mkApp {drv = packages.my-project;};
      defaultApp = apps.my-project;

      # `nix develop`
      devShell = pkgs.mkShell {
        buildInputs = da_packages;
        # supply the specific rust version
        nativeBuildInputs = [
          rust
          pkgs.rust-analyzer
          pkgs.git
          pkgs.cargo-udeps
          pkgs.cargo-audit
          pkgs.bacon
          pkgs.cargo-tauri
          pkgs.yarn
          pkgs.nodejs-19_x
          pkgs.dpkg
          pkgs.tree
          pkgs.nodePackages.typescript-language-server
        ];
        shellHook = ''
          export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
          export PATH=$PATH:$(pwd)/node_modules/.bin/
        '';
      };
    });
}
# {

