use std::{fmt::Display, path::Path, process::exit, time::Instant};

use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::*;

mod cli;
mod librpgmaker;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Decrypt { game_dir, output } => {
            let mut game = get_game_or_exit(&game_dir, !args.quiet);

            pretty_print_key(&game);

            let files = scan_or_exit(&mut game);
            let count = count_variants(files.iter());
            println!("\n{}", count);

            println!("Starting decryption...");

            let dec_start = Instant::now();
            let num_dec = match game.decrypt_all(&output.unwrap_or(OutputSettings::InPlace)) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("\nFailed to decrypt the game: {:?}", e);
                    exit(1);
                }
            };
            let taken = dec_start.elapsed();

            if num_dec > 0 {
                println!("Decrypted {} files in {:.2?}", num_dec, taken);
            } else {
                eprintln!("No decryptable files found.");
            }
        }

        Commands::Scan { game_dir } => {
            let mut game = get_game_or_exit(&game_dir, false);
            let files = scan_or_exit(&mut game);

            let count = count_variants(files.iter());
            println!("{}", count);
        }

        Commands::Key { game_dir } => {
            let game = get_game_or_exit(&game_dir, false);

            pretty_print_key(&game);
        }
    }
}

fn get_game_or_exit(dir: &Path, verbose: bool) -> RpgGame {
    RpgGame::new(dir, verbose).unwrap_or_else(|e| {
        eprintln!("Failed to open game dir: {:?}", e);
        exit(1);
    })
}

fn scan_or_exit(game: &mut RpgGame) -> Vec<RpgFileType> {
    match game.scan_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Failed to scan the game: {:?}", e);
            exit(1);
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
        audio: *counts.get(&RpgFileType::RpgAudio).unwrap_or(&0),
        video: *counts.get(&RpgFileType::RpgVideo).unwrap_or(&0),
        image: *counts.get(&RpgFileType::RpgImage).unwrap_or(&0),
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
        write!(
            f,
            "Found {} decryptable images, {} audios and {} videos",
            self.image, self.audio, self.video
        )
    }
}
