{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;
          nativeBuildInputs = with pkgs; [ rustToolchain ruby_3_3.devEnv ];
          # need libclang
          buildInputs = with pkgs; [
            libclang.lib
          ];
        in
        with pkgs;
        {
          devShells.default = mkShell {
            inherit buildInputs nativeBuildInputs;

            env.LIBCLANG_PATH = "${libclang.lib}/lib";
          };
        }
      );
}