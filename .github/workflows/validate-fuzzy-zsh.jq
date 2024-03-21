include "./.github/workflows/lib";

. as $output
| assert($output | length | . > 5; "expected several results")

| $output
| map(select(.attribute == "zsh"))
| . as $filtered
| assert($filtered | length | . == 1; "expected `zsh` to be in the results")
# the rest of the info about zsh is already tested in the --exact test

| empty
