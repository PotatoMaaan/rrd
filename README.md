# rust-rpgmv-decrypt (rrd)
rust-rpgmv-decrypt (rrd) is a small program that decrypts games made with rpgmaker-mv, written in Rust.

The aim is to be small, simple, fast and reliable.

## Bulding
```sh
cargo build --release #builds an optimized release bianry
```

## Usage
```
Usage: rrd [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>  The game directory containing the main executable file

Options:
  -k, --keep-original    Keep the original (encrypted) file next to the decrypted files
  -o, --output <OUTPUT>  The directory where decrypted files are output to relative to the current directory. This automatically keeps the encrypted files in place. If not specified, the files will be alongside the encrypted ones
  -h, --help             Print help
```
