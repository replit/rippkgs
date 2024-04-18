# Rippkgs

rippkgs is a search CLI utility for searching indexed nixpkgs expressions.

## Usage

### Installation

Once <https://github.com/NixOS/nixpkgs/pull/305125> is merged, if you have the commit you can use install using your preferred way of installing nixpkgs derivations.
For example:
```sh
$ nix profile install nixpkgs#rippkgs nixpkgs#rippkgs-index
# alternatively, via the flake directly
$ nix profile install github:replit/rippkgs/v1.1.0
```

Alternatively, if you're using a flake to configure your system, you can add rippkgs via nixpkgs or as an input to your flake and add rippkgs to your environment packages.
For example, in NixOS:
```nix
environment.systemPackages = [
  pkgs.rippkgs
  pkgs.rippkgs-index
  # alternatively, if via the flake input:
  inputs.rippkgs.packages.${system}.rippkgs
  inputs.rippkgs.packages.${system}.rippkgs-index
];
```

### Generate an index

Generating an index may be done with the `rippkgs-index` cli:
```sh
rippkgs-index nixpkgs -o $XDG_DATA_HOME/rippkgs-index.sqlite
```

If you don't have a `nixpkgs` channel set (or would prefer to index a different channel), you'll have to explicitly pass the dir to the nixpkgs distribution:
```sh
rippkgs-index nixpkgs -o $XDG_DATA_HOME/rippkgs-index.sqlite ~/.nix-defexpr/channels/my-very-special-nixpkgs-channel
```

Alternatively, you can generate a registry using the flake output `lib.genRegistry`, which allows you to avoid recursive-nix problems:
```sh
$ nix eval -L .#lib.genRegistry --apply 'f: f (import <nixpkgs> { })' --impure --json >registry.json
$ rippkgs-index registry -o rippkgs-index.sqlite registry.json
```

### Searching

Use the `rippkgs` cli to search for appropriate packages:
```sh
rippkgs rustc
```

## Comparison

`nix-env -q` is historically the command that's used to achieve what rippkgs achieves, but the nix evaluation cost is high.
When in an environment where reactiveness is desirable, it's better to pay the initial cost of generating the index (see results below).

[`nix-index`](https://github.com/nix-community/nix-index) is similar in that it operates on a generated database,
but requires derivations to be built in order to generate the index, and not all package information is stored in the index.
This means you have to augment `nix-locate` results with `nix-env` in order to get additional information about a package, like its
version or the nixpkgs registry description for the package.

<https://search.nixos.org> is the closest approximate tool to rippkgs, but unfortunately it comes with a few shortcomings:
1. The index only tracks up-to-date nixpkgs distributions, which means you *must* resort to `nix-env` to get results accurate to the nixpkgs release you're using.
2. Search is only available as a web service with only HTML responses to its HTTP api, which means you have to parse the HTML response to your programmatic requests.
3. It uses Elastic Search, which is fine for a widely-used service but doesn't work well when providing highly-localized results.

## Performance

Using [nixpkgs@b550fe4b4776908ac2a861124307045f8e717c8e](https://github.com/NixOS/nixpkgs/tree/b550fe4b4776908ac2a861124307045f8e717c8e) on an aarch64-darwin with 16gb of memory:
```sh
$ time ./target/release/rippkgs-index nixpkgs -o nixpkgs.sqlite
evaluated registry in 289.8638 seconds
parsed registry in 0.0761 seconds
wrote index in 0.1432 seconds
./target/release/rippkgs-index nixpkgs -o nixpkgs.sqlite  108.35s user 45.50s system 53% cpu 4:50.20 total

$ time ./target/release/rippkgs -i nixpkgs.sqlite rustc 1>/dev/null
got results in 44 ms
./target/release/rippkgs -i nixpkgs.sqlite rustc > /dev/null  0.03s user 0.02s system 89% cpu 0.051 total
```
