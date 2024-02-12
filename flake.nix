{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake {inherit inputs;} {
    imports = [
      inputs.devshell.flakeModule
    ];

    systems = [
      "aarch64-darwin"
      "x86_64-linux"
    ];

    perSystem = { lib, pkgs, rust-toolchain, self', system, ... }: {
      _module.args = {
        pkgs = import inputs.nixpkgs {
          inherit system;

          overlays = [
            inputs.fenix.overlays.default
          ];
        };

        rust-toolchain = pkgs.fenix.latest.toolchain;
      };

      formatter = pkgs.alejandra;

      devShells.default = self'.devShells.rippkgs;
      devshells.rippkgs = {
        packages = [
          pkgs.fenix.latest.toolchain
          pkgs.sqlite
        ];
      };

      packages = let
        craneLib = inputs.crane.lib.${system}.overrideToolchain rust-toolchain;

        common-args = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strict-deps = true;

          buildInputs = [
            pkgs.sqlite
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly common-args;
      in {
        default = self'.packages.rippkgs;

        rippkgs = craneLib.buildPackage (common-args // {
          inherit cargoArtifacts;
          pname = "rippkgs";
          # cargoExtraArgs = "--bin rippkgs";
        });

        rippkgs-index = craneLib.buildPackage (common-args // {
          inherit cargoArtifacts;
          pname = "rippkgs-index";
        });
      };
    };
  };
}
