{
  description = "Wormhole: a URL shortener service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    gomod2nix = {
      url = "github:nix-community/gomod2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
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
      gomod2nix,
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

        # helper to build Go applications using gomod2nix
        buildGoApplication = gomod2nix.legacyPackages.${system}.buildGoApplication;

        unfilteredRoot = ./.;
        src = craneLib.cleanCargoSource unfilteredRoot;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            protobuf
          ];

          buildInputs =
            [ ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
        };

        commonArgsExtra = commonArgs // {
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              (lib.fileset.fileFilter (file: file.hasExt "proto") unfilteredRoot)
              (lib.fileset.fileFilter (file: file.hasExt "sql") unfilteredRoot)
            ];
          };
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              (lib.fileset.fileFilter (file: file.hasExt "proto") unfilteredRoot)
              (lib.fileset.fileFilter (file: file.hasExt "sql") unfilteredRoot)
            ];
          };

        # gateway
        wormhole-gateway = craneLib.buildPackage (
          individualCrateArgs
          // {
            inherit cargoArtifacts;
            pname = "wormhole-gateway";
            cargoExtraArgs = "-p wormhole-gateway";
            src = fileSetForCrate ./crates/wormhole-gateway;
          }
        );
        # redirector service
        wormhole-redirector = craneLib.buildPackage (
          individualCrateArgs
          // {
            inherit cargoArtifacts;
            pname = "wormhole-redirector";
            cargoExtraArgs = "-p wormhole-redirector";
            src = fileSetForCrate ./crates/wormhole-redirector;
          }
        );
        # shortener service
        wormhole-shortener = craneLib.buildPackage (
          individualCrateArgs
          // {
            inherit cargoArtifacts;
            pname = "wormhole-shortener";
            cargoExtraArgs = "-p wormhole-shortener";
            src = fileSetForCrate ./crates/wormhole-shortener;
          }
        );

        wormhole-analytics = buildGoApplication {
          pname = "wormhole-analytics";
          version = "0.1.0";
          src = ./analytics;
          modules = ./analytics/gomod2nix.toml;
          subPackages = [ "cmd/server" ];
          go = pkgs.go_1_25;
          postInstall = ''
            mv $out/bin/server $out/bin/analytics
          '';
        };

        image = pkgs.dockerTools.buildLayeredImage {
          name = "wormhole";
          tag = "latest";

          contents = [
            wormhole-gateway
            wormhole-redirector
            wormhole-shortener
            pkgs.grpc-health-probe
          ];

          config = {
            Cmd = [ ];
            WorkingDir = "/tmp";
          };
        };
      in
      {
        checks = {
          inherit
            wormhole-redirector
            wormhole-shortener
            wormhole-gateway
            wormhole-analytics
            image
            ;

          wormhole-clippy = craneLib.cargoClippy (
            commonArgsExtra
            // {
              inherit cargoArtifacts;
              # we are in active development, so we don't want to fail
              # the check on warnings, but we still want to see them
              cargoClippyExtraArgs = "--all-targets";
              # cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          wormhole-doc = craneLib.cargoDoc (
            commonArgsExtra
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
          inherit
            wormhole-redirector
            wormhole-shortener
            wormhole-gateway
            wormhole-analytics
            image
            ;
        };
      }
    );
}
