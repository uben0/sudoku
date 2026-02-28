{
  description = "A dev shell with Rust nightly";

  inputs = {
         nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
     flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem ( system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bun
            pkgs.gleam
            pkgs.erlang
            pkgs.rebar3
            pkgs.wasm-tools
            (pkgs.rust-bin.selectLatestNightlyWith ( toolchain:
              toolchain.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "wasm32-unknown-unknown" ];
              }
            ))
          ];

          shellHook = "exec ${pkgs.fish}/bin/fish";
        };
      }
    );
}
