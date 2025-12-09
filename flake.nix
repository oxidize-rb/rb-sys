{
  description = "Dev environment for rb-sys";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            fastmod
            ruby_3_4.devEnv
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
              targets = [ "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" "aarch64-apple-darwin" ];
            })
            bundler
            zsh
            nodejs
            libiconv
            zig
            pkg-config
            zstd.dev
            llvmPackages.libclang
            lld
          ];

          # Environment setup for low-memory builds and macOS compatibility
          shellHook = ''
            # Use LLVM's lld (lower memory) for Rust linking
            export RUSTFLAGS="''${RUSTFLAGS:+$RUSTFLAGS }-C linker=clang -C link-arg=-fuse-ld=lld -C codegen-units=32"

            # Make libiconv visible to the linker and headers to the C compiler
            export LIBRARY_PATH="${libiconv}/lib:''${LIBRARY_PATH}"
            export CPATH="${libiconv}/include:''${CPATH}"

            # Make system zstd visible to zstd-sys to skip compiling zstd C sources
            export ZSTD_SYS_USE_PKG_CONFIG=1
            export PKG_CONFIG_PATH="${zstd.dev}/lib/pkgconfig:''${PKG_CONFIG_PATH}"

            # Bindgen/clang-sys – ensure libclang is resolvable and consistent
            export LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib"

            # Reduce concurrent rustc/C compiles to cut peak memory
            export CARGO_BUILD_JOBS=''${CARGO_BUILD_JOBS:-3}
          '';
        };
      }
    );
}
