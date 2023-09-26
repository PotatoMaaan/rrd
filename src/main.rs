use std::time::Instant;

use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::*;

mod cli;
mod librpgmaker;
mod util;

fn main() {
    let args = Cli::parse();

    let mut game = RpgGame::new(args.directory, !args.quiet).unwrap();

    println!("Scanning...");
    let files = game.scan_files().unwrap();
    let count = count_variants(files.iter());
    println!(
        "Found {} images, {} audios and {} videos",
        count.image, count.audio, count.video
    );

    let output_options = match (args.flatten_paths, args.output) {
        (true, None) => panic!("invalid args"),
        (true, Some(out_dir)) => OutputSettings::Flatten { dir: out_dir },
        (false, None) => OutputSettings::InPlace,
        (false, Some(out_dir)) => OutputSettings::Specific { dir: out_dir },
    };

    println!("Decrypting game...");
    let start_time = Instant::now();
    let num_dec = game.decrypt_all(&output_options).unwrap();

    println!(
        "Decrypted {} files in {:.2?}",
        num_dec,
        start_time.elapsed()
    );

    dbg!(args.quiet);
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
