use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::exit,
    time::Duration,
};

use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::{decode_hex, prelude::*};

mod cli;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::DecryptFile { path, output, key } => {
            let Some(file) = RpgFile::from_path_encrypted(&path) else {
                eprintln!("File is not a valid RpgMaker file!");
                exit(1);
            };

            let Ok(key) = librpgmaker::decode_hex(&key) else {
                eprintln!("Key is not valid!");
                exit(1);
            };
            let file = file.decrypt(&key).unwrap();

            if let Err(err) = fs::write(
                output.unwrap_or(file.decrypted_path().to_path_buf()),
                file.data(),
            ) {
                eprintln!("Failed writing file: {}", err);
            }
        }

        Commands::EncryptFile {
            ref path,
            key,
            output,
        } => {
            let file = RpgFile::from_path_decrypted(path).unwrap();

            let file = file.encrypt(&decode_hex(&key).unwrap()).unwrap();

            let path = output.unwrap_or(file.encrypted_path().to_path_buf());
            dbg!(&path);
            fs::write(path, file.data()).unwrap();
        }

        Commands::DecryptGame { path, output } => {
            let mut game = RpgGame::new(path, !args.quiet).unwrap();

            game.decrypt_all(&output.unwrap_or(OutputSettings::NextTo))
                .unwrap();
        }

        Commands::RestoreFile { ref path, output } => {
            let file = RpgFile::from_path_encrypted(path).unwrap();

            let file = file.restore_img().unwrap();

            let p = output.unwrap_or(file.decrypted_path().to_path_buf());
            dbg!(&p);
            fs::write(p, file.data()).unwrap();
        }

        Commands::Scan { path } => todo!(),

        Commands::Key { ref path } => {
            let game = get_game(path, &args);

            pretty_print_key(&game);
        }
    }
}

fn pretty_print_key(game: &RpgGame) {
    let key = game.get_key();

    if game.is_encrypted() {
        println!("The game is reporting that it is encrypted.");
    } else {
        println!("The game is reporting that it is NOT encrypted.");
    }

    println!("Found the following key:\n");

    println!("  Text : {}", key.string);
    println!("  Bytes: {:02X?}\n", key.bytes);
}

fn count_variants<'a>(items: impl Iterator<Item = &'a RpgFileType>) -> Counts {
    let counts = items.counts();

    Counts {
        audio: *counts.get(&RpgFileType::Audio).unwrap_or(&0),
        video: *counts.get(&RpgFileType::Video).unwrap_or(&0),
        image: *counts.get(&RpgFileType::Image).unwrap_or(&0),
    }
}

#[derive(Debug)]
struct Counts {
    audio: usize,
    video: usize,
    image: usize,
}

impl Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total = self.audio + self.video + self.image;
        write!(
            f,
            "Found {} decryptable items:\n\n   - images: {}\n   - audios: {}\n   - videos: {}\n",
            total, self.image, self.audio, self.video
        )
    }
}

/// Averages the duration of the successfull decryption results
fn avg_durations<'a>(durations: &[(Duration, PathBuf)]) -> Duration {
    let len = durations.len() as u32;
    let total: Duration = durations.iter().map(|(dur, _)| dur).sum();

    total / len
}

fn split_results<V, E>(results: Vec<Result<V, E>>) -> (Vec<V>, Vec<E>) {
    // Ther's probably a cleaner way to split the results, but oh well...
    let mut succeeded = Vec::with_capacity(results.len());
    let failed = results
        .into_iter()
        .filter_map(|x| match x {
            Ok(v) => {
                succeeded.push(v);
                None
            }
            Err(e) => Some(e),
        })
        .collect::<Vec<_>>();

    (succeeded, failed)
}

fn get_game(path: &Path, args: &cli::Cli) -> RpgGame {
    match RpgGame::new(path, !args.quiet) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open game: {}", e);
            exit(1);
        }
    }
}
