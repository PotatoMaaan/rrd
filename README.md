# rust-rpgmv-decrypt (rrd)

rust-rpgmv-decrypt (rrd) is a small program that decrypts games made with rpgmaker-mv, written in Rust.

The repo consists of the crate `rrd`, which is the actual binary program, while the `librpgmaker` crate provides a library to interact with rpgmaker games, which `rrd` uses.

### Credits

- Based on [the minimal example](https://bitbucket.org/SilicaAndPina/rpgmv-decryptor/src/master/) by [SilicaAndPina on bitbucket.org](https://bitbucket.org/SilicaAndPina/)

## Basic usage

1. First, download an existing binary or [build one yourself](#Building)
   (see the building section).
2. Drag and drop the folder you want to decrypt onto the executable or run the executable from a terminal.

## Advanced usage

```
Decrypt files encrypted by RPMVs default encryprion

Usage: rrd [OPTIONS] <GAME_DIR> [COMMAND]

Commands:
  next-to  Decrypts the game's files next to the encrypted files
  replace  Overwrites the games files with the decrypted ones
  output   Leaves the game untouched, places files into given directory while maintining original dir structure
  flatten  Same as output but flattens the dir structure
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <GAME_DIR>  The game directory

Options:
  -q, --quiet    Don't print individual files during decryption
  -s, --scan     Just scan the amount of decryptable files
  -k, --key      Just print the key
  -h, --help     Print help
  -V, --version  Print version

```

## Building

To build `rrd` you just need the [the rust toolchain](https://rustup.rs/) and git.

```sh
git clone https://github.com/PotatoMaaan/rrd.git
cd rrd
cargo build --release #builds an optimized release binary in target/release
```

### Note

This is only intended for local modding etc. Don't steal assets!
