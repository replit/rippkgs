include "./.github/workflows/lib";

. as $output
| assert($output | length | . == 1; "no results")

| $output[0] as $zsh
| assertField($zsh; "attribute"; "zsh")
| assertField($zsh; "name"; "zsh")
| assertField($zsh; "version"; "5.9")
| assertField($zsh; "description"; "The Z shell")
| assert($zsh | .long_description | startswith("Zsh is a UNIX command interpreter (shell)"); "wrong long_description")
| assert($zsh | has("store_path"); "expected ")

| empty
