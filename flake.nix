{
  description = "Glint - Local-first observability for developers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-bloat
            just
            mold
            clang
            lld
            protobuf
            bun
            sccache
          ];

          shellHook = ''
            echo "Glint Development Environment"
            echo "Rust:    $(rustc --version)"
            echo "Cargo:   $(cargo --version)"
            echo "Protoc:  $(protoc --version)"
            echo "Linker:  mold (fastest Rust linker)"
            echo ""
            echo "Quick commands:"
            echo "  cargo build           - Fast dev build"
            echo "  cargo build --release - Optimized release build"
            echo "  cargo test            - Run tests"
            echo ""
          '';
        };
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "glint";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = with pkgs; [
            protobuf
            mold
            clang
          ];
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
        };
      }
    );
}
