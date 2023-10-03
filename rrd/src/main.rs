use std::{
    fmt::Display,
    path::PathBuf,
    process::exit,
    time::{Duration, Instant},
};

use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::prelude::*;

mod cli;

fn main() {
    let args = Cli::parse();

    let mut game = RpgGame::new(args.game_dir, !args.quiet).unwrap_or_else(|e| {
        eprintln!("Failed to open game dir: {}", e);
        exit(1);
    });

    pretty_print_key(&game);

    if args.key {
        exit(0);
    }

    let scanned = match game.scan_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Failed to scan the game: {}", e);
            exit(1);
        }
    };
    let counts = count_variants(scanned.iter());
    println!("{}", counts);

    if args.scan {
        exit(0);
    }

    println!("Starting decryption...");
    let start_time = Instant::now();
    let results = match game.decrypt_all(&args.output.unwrap_or(OutputSettings::NextTo)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to decryptt the game: {}", e);
            exit(1);
        }
    };

    let (succeeded, failed) = split_results(results);

    println!("\n");
    if !failed.is_empty() {
        println!("\n");

        for error in &failed {
            eprintln!("ERROR: {}", error);
        }
        print!(
            "\n{} errors were encountered while decrypting",
            failed.len()
        );
    } else {
        println!("Game decrypted sucessfully!")
    }

    println!(
        "\nDecrypted {}/{} files in {:.2?}",
        succeeded.len(),
        scanned.len(),
        start_time.elapsed()
    );

    if succeeded.iter().count() > 1 {
        let avg = avg_durations(&succeeded);
        let max = succeeded
            .iter()
            .max_by(|(a, _), (b, _)| a.cmp(b))
            .expect("iter empty");

        println!("   -> Average time per item: {:.2?}", avg);
        println!(
            "   -> The file '{}' took the longest at {:.2?}\n",
            max.1.display(),
            max.0
        );
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
