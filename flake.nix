{
  description = "C/C++ environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        # This is a derivation to be used downstream.
        zermio-cli = pkgs.rustPlatform.buildRustPackage rec {
          pname = "zermio-cli";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = "${src}/Cargo.lock";
          };

          doCheck = false;
          meta = {
            description = "A CLI tool to generate mmio code from hardware descripition languages";
            homepage = "https://github.com/engdoreis/zermio";
          };
        };
      in {
        formatter = pkgs.alejandra;
        packages.default = zermio-cli;
        apps.default = flake-utils.lib.mkApp {
          drv = zermio-cli;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustup
            pkg-config
            openssl
            gdb
          ];
          shellHook = ''
            echo "Rust dev environment ready!"
          '';
        };
      }
    );
}
