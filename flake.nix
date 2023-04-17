{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          inherit (pkgs) rustPlatform;
          package = rustPlatform.buildRustPackage {
            name = "classicube-anti-afk-plugin";
            src = lib.cleanSourceWith rec {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  let
                    baseName = builtins.baseNameOf (builtins.toString path);
                    relPath = lib.removePrefix (builtins.toString ./.) (builtins.toString path);
                  in
                  lib.any (re: builtins.match re relPath != null) [
                    "/Cargo.toml"
                    "/Cargo.lock"
                    "/src"
                    "/src/.*"
                  ]
                );
            };
            cargoSha256 = "sha256-rcs8DUvxiHBamOh5h5IwH7VYtzWUOeY0ubArsVGyJpg=";
            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ];
            buildInputs = with pkgs; [
              alsa-lib
              at-spi2-atk
              cairo
              gdk-pixbuf
              glib
              gtk3
              openssl
              pango
            ];

            doCheck = false;
          };
        in
        rec {
          devShells.${system}.default = package.overrideAttrs (old: {
            nativeBuildInputs = with pkgs; old.nativeBuildInputs ++ [
              clippy
              rustfmt
              rust-analyzer
            ];
          });
          packages.${system}.default = package;
        }
      )
      lib.systems.flakeExposed);
}
