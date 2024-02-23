# Rippkgs

rippkgs is a search CLI utility for searching indexed nixpkgs distributions.

## Usage

First, an index must be generated. Doing so requires the `rippkgs-index` cli:
```sh
rippkgs --nixpkgs '/path/to/nixpkgs/dir' --nixpkgs-config '<nixpkgs/pkgs/top-level/packages-config.nix>' -o index.sqlite
```

Afterwards, use the `rippkgs` cli to search for appropriate packages:
```sh
rippkgs --index index.sqlite rustc
```

