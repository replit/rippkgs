{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      perSystem = {
        config,
        craneLib,
        crane-common-args,
        cargoArtifacts,
        lib,
        pkgs,
        rust-toolchain,
        self',
        system,
        ...
      }: {
        _module.args = {
          rust-toolchain = inputs.fenix.packages.${system}.stable.toolchain;
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rust-toolchain;

          crane-common-args = {
            src = ./.;
            strict-deps = true;

            buildInputs =
              [
                pkgs.sqlite
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                pkgs.libiconv
              ];
          };

          cargoArtifacts = craneLib.buildDepsOnly crane-common-args;
        };

        checks = {
          inherit (self'.packages) rippkgs rippkgs-index;

          clippy = craneLib.cargoClippy (crane-common-args
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

          cargo-fmt = craneLib.cargoFmt {
            src = ./.;
          };

          cargo-nextest = craneLib.cargoNextest (crane-common-args
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            });

          nix-fmt = pkgs.runCommand "nix-fmt-check" {} ''
            cd ${./.}
            ${pkgs.alejandra}/bin/alejandra --check . && mkdir -p $out
          '';
        };

        formatter = pkgs.alejandra;

        devShells.default = self'.devShells.rippkgs;
        devShells.rippkgs = pkgs.mkShell {
          packages = [
            rust-toolchain
            pkgs.alejandra
            pkgs.jq
            pkgs.sqlite
          ];
        };

        overlayAttrs = {
          inherit (config.packages) rippkgs rippkgs-index;
        };

        packages = {
          default = self'.packages.rippkgs;

          rippkgs = craneLib.buildPackage (crane-common-args
            // {
              inherit cargoArtifacts;
              pname = "rippkgs";
              cargoExtraArgs = "--bin rippkgs";
              meta.mainProgram = "rippkgs";
            });

          rippkgs-index = craneLib.buildPackage (crane-common-args
            // {
              inherit cargoArtifacts;
              pname = "rippkgs-index";
              cargoExtraArgs = "--bin rippkgs-index";
              meta.mainProgram = "rippkgs-index";
            });
        };
      };

      flake = args: {
        lib = import ./lib args;
      };
    };
}
