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
    protobuf
    protoc-gen-go
    protoc-gen-go-grpc
    # golang
    sqlc
    goose
    # useful tools
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
        ${pkgs.protobuf}/bin/protoc proto/**/*.proto \
            --go_out=./analytics/pb \
            --go-grpc_out=./analytics/pb \
            --go_opt=paths=source_relative \
            --go-grpc_opt=paths=source_relative \
            --proto_path=proto # define the proto path for imports
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
      update-sqlc-generated = {
        enable = true;
        entry = "${lib.getExe pkgs.sqlc} generate -f analytics/sqlc.yaml";
        files = "analytics\/sqlc\/.*\.sql";
        pass_filenames = false;
      };
    };
    package = pkgs.prek;
  };

  # See full reference at https://devenv.sh/reference/options/
}
