{
  description = "C/C++ environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
        fenixLib = fenix.packages.${system};
        rust_toolchain = fenixLib.fromToolchainFile {
          dir = ./.;
          sha256 = "sha256-X/4ZBHO3iW0fOenQ3foEvscgAPJYl2abspaBThDOukI=";
        };

        zermio-cli = pkgs.callPackage ./default.nix {};
      in {
        formatter = pkgs.alejandra;
        packages.default = zermio-cli;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust_toolchain
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
