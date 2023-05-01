# rust-rpgmv-decrypt (rrd)
rust-rpgmv-decrypt (rrd) is a small program that decrypts games made with rpgmaker-mv, written in Rust.

The aim is to be small, simple, fast and reliable.

Sice the recent implimentation of async file decryption, this might just be the fastest rpgmv decrypter out there :) (please open pr with correction if you find anything faster!)

Here is a comparision (file cache in ram was cleard before earch run)
- Old implimentation took 36 seconds
- Async implementation took 14 seconds


### Credits
A lot of the algorythim comes from [here](https://bitbucket.org/SilicaAndPina/rpgmv-decryptor/src/master/)

### Notes for Windows users
- Windows defender may detect the file as malicious in some way. (When testing it, i got "Windows protected your PC") If you don't trust the biaries because of this, you can easily build the program yourself (see the building section)
- On windows, it's theoretically enough to drag the game folder onto the downloaded program, however, this won't give you any access to options and means that you always need to keep the file around somewhere where you can find it. Because of that, I recommend you do the following:

## Installation

1. First, download an existing binary or compile the program yourself. (see the building section)
2. Rename the file to whatever you want
3. Place the file into a directory which is in your PATH. (for example `C:\Windows` on Windows or `/usr/local/bin` on most UNIX-like Systems.
4. Open a terminal and type the name you gave your file in step 1.

## Example installation and use (Windows)
1. Do the steps from the installation section, your file (in this case named rrd.exe) should now be in `C:\Windows`.
2. In explorer, navigate to he folder where your game is, in this example, it is a folder named `game` inside the `Documents` folder. We don't actually want to go into the `game` folder, but rather the folder that contains the game, in this case, `Documents`.
3. Click into the url bar in Explorer and type `cmd`

![drt](/tutorial-images/example-url.png)
![dgdf](/tutorial-images/example-launch-cmd.png)

4. This will open a command prompt window. Now type the name you just gave your file, in this case `rrd`. you should see the following output:

![dgdf](/tutorial-images/example-command-1.png)

5. This is telling us to provide a directory (folder). Since we want to decrypt the `game` folder, we type `rrd game`. In the example command below, the `-o` option is also set to `decrypted` which will create a folder `decryped` and put all the decrypted files in there. (this is entirely optional)

![dgdf](/tutorial-images/example-command-2.png)

6. Press ENTER and wait for the decryption to finish. Once finished, you should see something similar to the following output:

![dgdf](/tutorial-images/example-finished.png)

## Bulding
Building rust programs (such as this) is very simple. You only need to install [the Rust toolchain](https://rustup.rs/) for your system and execute the following commands:
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
