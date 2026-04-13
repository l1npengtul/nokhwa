{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
          config.allowUnfree = true;
        };
        rustshell = pkgs.mkShell.override {
          stdenv = pkgs.gccStdenv;
        };
        rustbin = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
            toolchain.default.override {
              extensions = [
                "rust-src"
                "clippy"
                "rustfmt"
                "miri"
                "rust-analyzer"
              ];
            }
        );
      in {
        formatter = pkgs.alejandra;

        devShells.default = rustshell {
          packages =
            [
              rustbin
            ]
            ++ (with pkgs; [
              llvmPackages_21.clangWithLibcAndBasicRtAndLibcxx
              pkg-config
              cmake
              vcpkg
              lldb
              rustPlatform.bindgenHook
              xmlstarlet
              opencv
              alsa-lib
              systemdLibs
              cmake
              fontconfig
              linuxHeaders
              v4l-utils
              libv4l
              pipewire
              rustup
              gcc
              ffmpeg_8-full
              nasm
              libGL
              flite
              quirc
              lcevcdec
              xz
              celt
              opencore-amr
              snappy
              codec2
              gsm
              ilbc
              lame
              libtheora
              libogg
              twolame
              vo-amrwbenc
              vvenc
              xavs
              xvidcore
              soxr
              libvdpau
              gmp
              openapv
              svt-av1
              mkvtoolnix-cli
            ]);

          env.RUST_SRC_PATH = "${rustbin}/lib/rustlib/src/rust/library";
          env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          shellHook = let
            pathToRustProject = "/project/component[@name='RustProjectSettings']";
          in ''
            echo "WONDERHOOOOOY!!!!"
            xmlstarlet edit --inplace --update "${pathToRustProject}/option[@name='explicitPathToStdlib']/@value" --value "${rustbin}/lib/rustlib/src/rust/library" .idea/workspace.xml
            xmlstarlet edit --inplace --update "${pathToRustProject}/option[@name='toolchainHomeDirectory']/@value" --value "${rustbin}/bin" .idea/workspace.xml
          '';
        };
      }
    );
}
