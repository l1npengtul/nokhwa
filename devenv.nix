{ pkgs, lib, config, inputs, stdenv, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.LIBCLANG_PATH = "${config.env.DEVENV_PROFILE}/lib/libclang.so";
  env.CPATH = "${config.env.DEVENV_PROFILE}/include";
  # https://devenv.sh/packages/
  packages = [ pkgs.git (pkgs.v4l-utils.override { withUtils = true; }) pkgs.clangStdenv pkgs.mesa
      pkgs.cmake pkgs.opencv4 pkgs.systemdLibs pkgs.libudev-zero
      pkgs.libudev0-shim pkgs.vcpkg pkgs.pkg-config pkgs.libclang
      pkgs.fontconfig pkgs.clang-tools pkgs.linuxHeaders pkgs.gccStdenv pkgs.libcxxStdenv
      pkgs.glibc_multi
  ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;
  # https://devenv.sh/processes/
  processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  enterShell = ''
    hello
    git --version
    echo ''${LIBCLANG_PATH}
    echo ''${CPATH}
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep --color=auto "${pkgs.git.version}"
  '';



  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # See full reference at https://devenv.sh/reference/options/
}