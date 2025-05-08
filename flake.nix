{
    description = "C/C++ environment";

    inputs = {
         nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
         flake-utils.url = "github:numtide/flake-utils";
    };

outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
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
