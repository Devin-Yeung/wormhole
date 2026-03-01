{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  env.PROTOC = lib.getExe pkgs.protobuf;

  dotenv.disableHint = true;

  overlays = [
    inputs.gomod2nix.overlays.default
  ];

  # https://devenv.sh/packages/
  packages = with pkgs; [
    cargo-deny
    cargo-machete
    cargo-nextest
    cargo-modules
    sqlx-cli
    ijhttp
    hurl
    grpc-health-probe
    grpcurl
    protobuf
    protoc-gen-go
    protoc-gen-go-grpc
    gomod2nix
    kcat
    iredis
    mycli
    lnav
    just
  ];

  # https://devenv.sh/languages/
  languages = {
    rust = {
      enable = true;
      toolchainFile = ./rust-toolchain.toml;
    };
  };

  # https://devenv.sh/scripts/
  enterShell = ''
    rustc --version
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
  '';

  # https://devenv.sh/pre-commit-hooks/
  git-hooks = {
    hooks = {
      clippy = {
        packageOverrides = {
          cargo = config.languages.rust.toolchainPackage;
          clippy = config.languages.rust.toolchainPackage;
        };
        enable = true;
      };
      rustfmt = {
        packageOverrides = {
          cargo = config.languages.rust.toolchainPackage;
          rustfmt = config.languages.rust.toolchainPackage;
        };
        enable = true;
      };
      nixfmt.enable = true;
      taplo.enable = true;
      yamlfmt = {
        enable = true;
        settings = {
          lint-only = false;
        };
      };
      trim-trailing-whitespace.enable = true;
    };
    package = pkgs.prek;
  };

  # See full reference at https://devenv.sh/reference/options/
}
