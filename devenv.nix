{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{

  imports = [
    ./nix/go/gomod2nix.nix
  ];

  env.PROTOC = lib.getExe pkgs.protobuf;

  dotenv.disableHint = true;

  # https://devenv.sh/packages/
  packages = with pkgs; [
    # rust tools
    cargo-deny
    cargo-machete
    cargo-nextest
    cargo-modules
    sqlx-cli
    # http tools
    ijhttp
    hurl
    # grpc tools
    grpc-health-probe
    grpcurl
    buf
    protobuf
    protoc-gen-go
    protoc-gen-go-grpc
    # golang
    sqlc
    goose
    # useful tools
    zensical # for docs
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
    update-go-pb = {
      description = "update generated Go protobuf files";
      exec = ''
        ${pkgs.buf}/bin/buf generate
      '';
    };
  };

  # https://devenv.sh/pre-commit-hooks/
  git-hooks = {
    hooks = {
      clippy = {
        enable = true;
      };
      rustfmt = {
        enable = true;
      };
      nixfmt.enable = true;
      taplo = {
        enable = true;
      };
      yamlfmt = {
        enable = true;
        settings = {
          lint-only = false;
        };
      };
      trim-trailing-whitespace.enable = true;
      update-go-pb = {
        enable = true;
        entry = "update-go-pb";
        files = "\\.proto$";
        pass_filenames = false;
      };
      buf-lint = {
        enable = true;
        entry = "${pkgs.buf}/bin/buf lint";
        files = "\\.proto$";
        pass_filenames = false;
      };
      update-sqlc-generated = {
        enable = true;
        entry = "${lib.getExe pkgs.sqlc} generate -f services/analytics/sqlc.yaml";
        files = "services\/analytics\/sqlc\/.*\.sql";
        pass_filenames = false;
      };
    };
    package = pkgs.prek;
  };

  # See full reference at https://devenv.sh/reference/options/
}
