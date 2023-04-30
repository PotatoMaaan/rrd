# rust-rpgmv-decrypt (rrd)
rust-rpgmv-decrypt (rrd) is a small program that decrypts games made with rpgmaker-mv, written in Rust.

The aim is to be small, simple, fast and reliable.

Sice the recent implimentation of async file decryption, this might just be the fastest rpgmv decrypter out there :) (please open pr with correction if you find anything faster!)

Here is a comparision (file cache in ram was cleard before earch run)
- Old implimentation took 36 seconds
- Async implementation took 14 seconds


### Credits
A lot of the algorythim comes from [here](https://bitbucket.org/SilicaAndPina/rpgmv-decryptor/src/master/)

## Installation
1. First, download an existing binary or compile the program yourself. (see the building section)
2. Rename the file to whatever you want
3. Place the file into a directory which is in your PATH. (for example `C:\Windows\System32` on Windows or `/usr/local/bin` on most UNIX-like Systems.
4. Open a terminal and type the name you gave your file in step 1.

## Bulding
```sh
git clone https://github.com/PotatoMaaan/rrd.git
cd rrd
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
### Note
This is only intended for local modding etc. Don't steal assets!
