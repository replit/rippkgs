#!/usr/bin/env bash

set -Eeuo pipefail
set -x

dir=$(mktemp -d)

nix eval .#lib.genRegistry --show-trace --apply 'f: f (import <nixpkgs> { config = import <nixpkgs/pkgs/top-level/packages-config.nix> // { allowUnsupportedSystem = true; }; })' --impure --json | jq . >$dir/lib-call-result.json

# nix-env --json -f '<nixpkgs>' -qa --meta --out-path | jq . >$dir/nix-env-result.json
nix run .#rippkgs-index -- -n '<nixpkgs>' -r $dir/nix-env-result.result.json -c 'import <nixpkgs/pkgs/top-level/packages-config.nix>' -o $(mktemp)
jq . $dir/nix-env-result.result.json >$dir/nix-env-result.json
rm $dir/nix-env-result.result.json

diff $dir/* >/dev/null

