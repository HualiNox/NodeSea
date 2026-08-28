{
  description = "NodeSea temporary development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          # Rust
          cargo
          rustc

          # Native build
          cmake
          clang
          pkg-config
          git

          # Native dependencies
          boost
          openssl
        ];

        # CMake FindOpenSSL and pkg-config hints.
        OPENSSL_ROOT_DIR = "${pkgs.openssl.out}";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        OPENSSL_CRYPTO_LIBRARY = "${pkgs.openssl.out}/lib/libcrypto.so";
        OPENSSL_SSL_LIBRARY = "${pkgs.openssl.out}/lib/libssl.so";
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

        shellHook = ''
          echo "NodeSea dev shell"
          echo "OpenSSL: ${pkgs.openssl.version}"
          echo "OPENSSL_ROOT_DIR=$OPENSSL_ROOT_DIR"
        '';
      };
    };
}
