# rustdisk

A `du`-style disk usage tool. Shows directory sizes as a tree, sorted by depth and size.
Further options to sort with different fields is planned. Parallelisation with rayon is also planned but less certain.

## Build

```bash
cargo build --release
./target/release/rustdisk [OPTIONS] [PATH]
```

## Usage

```
Usage: rustdisk [OPTIONS] [PATH]

Arguments:
  [PATH]  The target directory path [default: ./]

Options:
  -l, --level <LEVEL>
          How much depth to show. [default: 5]
      --shorten
          Shorten the output, control with width option
  -w, --width <WIDTH>
          Length to print out the file/dir name. [default: 20]
  -d, --dir-only
          Show directories only
      --show-percent-only
          Show percent of whole
      --show-size-only
          Show storage size
      --generate-completions <GENERATE_COMPLETIONS>
          Generate shell completions [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Example

```
$ rustdisk ./src --level 2
2026-06-15 22:11:14.888534 +07:00
----------------------------------
./src 20.000 KB
    tree.rs 8.000 KB
    error.rs 4.000 KB
    hrsize.rs 4.000 KB
    main.rs 4.000 KB
```

## How it works

Walks the directory tree iteratively with a stack. Each node accumulates its children's sizes on the way up. Output is sorted by depth then size descending.

Show size and percent by defautl but configurable.

Symlinks are not followed.

## Tab Completion Gneration

You can generate tab completion for options using `--generate-completions`.
Support zsh, bash, elvish, fish, and powershell.

For example, in zsh,

```bash
rustdisk --generate-completions=zsh > ~/.zsh/completions/_rustdisk
```

You may need to include ~/.zsh/completions in your FPATH for zsh. [more on FPATH](https://zsh.sourceforge.io/Doc/Release/Functions.html)
Consult your shell documetaion for tab completion functionalities.
