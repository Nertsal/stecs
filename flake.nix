{
  description = "ECS Flake";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        libDeps = with pkgs; [
            # cmake
            # fontconfig
            # mesa
            # freeglut
        ];
        libPath = pkgs.lib.makeLibraryPath libDeps;
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = libDeps ++ [
            # gcc
            # openssl
            # pkg-config
            # rust-bin.stable.latest.default
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ ];
            })
          ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${libPath}"
            export WINIT_UNIX_BACKEND=x11
          '';
        };
      }
    );
}
