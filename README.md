# rustdisk

A `du`-style disk usage tool. Shows directory sizes as a tree, sorted by depth and size.
Further options to sort with different fields is planned. Parallelisation with rayon is also planned but less certain.

## Build

```bash
cargo build --release
./target/release/rustdisk [PATH] [OPTIONS]
```

## Usage

```
Usage: rustdisk [OPTIONS] [PATH]

Arguments:
  [PATH]  The target directory path [default: ./]

Options:
  -l, --level <LEVEL>  How much depth to show. [default: 5]
  -s, --shorten        Shorten the output, control with width option
  -w, --width <WIDTH>  Length to print out the file/dir name. [default: 20]
  -d, --dir-only       Show directories only
  -h, --help           Print help
  -V, --version        Print version
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

Symlinks are not followed.
