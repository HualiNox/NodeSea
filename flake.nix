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
      llvm = pkgs.llvmPackages;
    in {
      devShells.${system}.default = pkgs.mkShell.override {
        stdenv = llvm.stdenv;
      } {
        nativeBuildInputs = with pkgs; [
          cargo
          rustc
          cmake
          llvm.clang
          llvm.lld
          llvm.llvm
          pkg-config
          git
        ];

        buildInputs = with pkgs; [
          boost
          boost.dev
          openssl
          openssl.dev
          stdenv.cc.cc.lib
        ];

        shellHook = ''
          export CC="${llvm.clang}/bin/clang"
          export CXX="${llvm.clang}/bin/clang++"
          export AR="${llvm.llvm}/bin/llvm-ar"
          export RANLIB="${llvm.llvm}/bin/llvm-ranlib"
          export LDFLAGS="-fuse-ld=lld"
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="${llvm.clang}/bin/clang"
          export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=lld -C link-arg=-Wl,-rpath,${pkgs.openssl.out}/lib -C link-arg=-Wl,-rpath,${pkgs.stdenv.cc.cc.lib}/lib"

          export BOOST_ROOT="${pkgs.boost.dev}"
          export BOOST_INCLUDEDIR="${pkgs.boost.dev}/include"
          export NODESEA_USE_SYSTEM_BOOST=1

          export OPENSSL_ROOT_DIR="${pkgs.openssl.out}"
          export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
          export OPENSSL_CRYPTO_LIBRARY="${pkgs.openssl.out}/lib/libcrypto.so"
          export OPENSSL_SSL_LIBRARY="${pkgs.openssl.out}/lib/libssl.so"
          export CMAKE_PREFIX_PATH="${pkgs.boost.dev}:${pkgs.openssl.dev}"
          export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
        '';
      };
    };
}
