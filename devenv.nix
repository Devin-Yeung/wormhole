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

  scripts = {
    update-gomod2nix = {
      description = "update gomod2nix.toml file";
      exec = ''
        for file in "$@"; do
          dir=$(dirname "$file")
          ${pkgs.gomod2nix}/bin/gomod2nix generate --dir "$dir"
        done
      '';
    };
  };

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
      taplo = {
        enable = true;
        excludes = [
          "analytics/gomod2nix.toml"
        ];
      };
      yamlfmt = {
        enable = true;
        settings = {
          lint-only = false;
        };
      };
      trim-trailing-whitespace.enable = true;
      update-gomod2nix = {
        enable = true;
        entry = "update-gomod2nix";
        files = "go.mod$";
      };
    };
    package = pkgs.prek;
  };

  # See full reference at https://devenv.sh/reference/options/
}
