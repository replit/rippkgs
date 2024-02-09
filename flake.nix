{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

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

    perSystem = { inputs', pkgs, ... }: {
      formatter = pkgs.alejandra;

      devshells.default = {
        packages = [
          inputs'.fenix.packages.latest.toolchain
          pkgs.sqlite
        ];
      };
    };
  };
}
