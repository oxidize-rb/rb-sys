{
  description = "Dev environment for rb-sys";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-ruby.url = "github:bobvanderlinden/nixpkgs-ruby";
    nixpkgs-ruby.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixpkgs-ruby, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        ruby = nixpkgs-ruby.lib.packageFromRubyVersionFile {
          file = ./.ruby-version;
          inherit system;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        packages.ruby = ruby;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.fastmod
            ruby
            rustToolchain
            pkgs.zsh
            pkgs.nodejs
          ];


        };
      }
    );
}
