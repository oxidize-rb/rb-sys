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
            ruby_3_3.devEnv
            rust-bin.stable.latest.default
            bundler
            zsh
            nodejs
          ];

          # Make is so nix develop --impure uses zsh config
          shellHook = ''
            exec zsh
          '';
        };
      }
    );
}
