{
  description = "Wormhole: a URL shortener service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
        inherit (pkgs) lib;

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "proto") unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "sql") unfilteredRoot)
          ];
        };

        commonArgs = {
          inherit src;
          pname = "wormhole";
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            protobuf
          ];

          buildInputs = lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # redirector service
        wormhole-redirector = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "wormhole-redirector";
            cargoExtraArgs = "-p wormhole-redirector";
          }
        );
        # shortener service
        wormhole-shortener = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "wormhole-shortener";
            cargoExtraArgs = "-p wormhole-shortener";
          }
        );
      in
      {
        checks = {
          inherit wormhole-redirector wormhole-shortener;

          wormhole-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              # we are in active development, so we don't want to fail the check on warnings, but we still want to see them
              cargoClippyExtraArgs = "--all-targets";
              # cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          wormhole-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          wormhole-fmt = craneLib.cargoFmt {
            inherit src;
          };

          wormhole-toml-fmt = craneLib.taploFmt {
            src = lib.sources.sourceFilesBySuffices src [ ".toml" ];
          };

          # todo: re-enable this when we split unit tests and integration tests
          # wormhole-nextest = craneLib.cargoNextest (
          #   commonArgs
          #   // {
          #     inherit cargoArtifacts;
          #     partitions = 1;
          #     partitionType = "count";
          #     cargoNextestPartitionsExtraArgs = "--no-tests=pass";
          #   }
          # );
        };

        packages = {
          inherit wormhole-redirector wormhole-shortener;
        };
      }
    );
}
