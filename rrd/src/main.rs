use std::{fmt::Display, process::exit, time::Instant};

use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::prelude::*;

mod cli;

fn main() {
    let args = Cli::parse();

    let mut game = RpgGame::new(args.game_dir, !args.quiet).unwrap_or_else(|e| {
        eprintln!("Failed to open game dir: {:?}", e);
        exit(1);
    });

    pretty_print_key(&game);

    if args.key {
        exit(0);
    }

    let scanned = match game.scan_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Failed to scan the game: {:?}", e);
            exit(1);
        }
    };
    let counts = count_variants(scanned.iter());
    println!("{}", counts);

    if args.scan {
        exit(0);
    }

    let start_time = Instant::now();
    let results = match game.decrypt_all(&args.output.unwrap_or(OutputSettings::NextTo)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to decryptt the game: {:?}", e);
            exit(1);
        }
    };
    let results_len = results.len();

    let failed = results
        .into_iter()
        .filter_map(|x| x.err())
        .collect::<Vec<_>>();

    println!("\n");
    if !failed.is_empty() {
        println!(
            "{} errors were encountered while decrypting:\n",
            failed.len()
        );

        for error in &failed {
            eprintln!("{:?}", error);
        }
    } else {
        println!("Game decrypted sucessfully!")
    }

    println!(
        "Decrypted {}/{} files in {:.2?}",
        results_len - failed.len(),
        scanned.len(),
        start_time.elapsed()
    );
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
