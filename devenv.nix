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
          # format the generated file
          ${lib.getExe pkgs.taplo} fmt "$dir/gomod2nix.toml"
        done
      '';
    };
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
      update-gomod2nix = {
        enable = true;
        entry = "update-gomod2nix";
        files = "go.mod$";
      };
      update-go-pb = {
        enable = true;
        entry = "update-go-pb";
        files = "\\.proto$";
        pass_filenames = false;
      };
    };
    package = pkgs.prek;
  };

  # See full reference at https://devenv.sh/reference/options/
}
