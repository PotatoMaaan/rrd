use clap::Parser;
use cli::*;
use librpgmaker::*;

mod cli;
mod librpgmaker;
mod util;

#[derive(Debug)]
struct Counts {
    audo: usize,
    video: usize,
    img: usize,
}

fn main() {
    let args = Cli::parse();

    let game = RpgGame::new(args.directory).unwrap();

    println!("Scanning...");
    let files = game.scan_files().unwrap();
    let counts = count_types(&files);

    println!("Found:\n{:#?}", counts);

    let output_options = match (args.flatten_paths, args.output) {
        (true, None) => panic!("invalid args"),
        (true, Some(out_dir)) => OutputSettings::Flatten { dir: out_dir },
        (false, None) => OutputSettings::InPlace,
        (false, Some(out_dir)) => OutputSettings::Specific { dir: out_dir },
    };

    game.decrypt_all(&output_options).unwrap();
}

fn count_types(files: &[RpgFileType]) -> Counts {
    let num_audio = files
        .iter()
        .filter(|t| t == &&RpgFileType::RpgAudio)
        .count();
    let num_video = files
        .iter()
        .filter(|t| t == &&RpgFileType::RpgVideo)
        .count();
    let num_img = files
        .iter()
        .filter(|t| t == &&RpgFileType::RpgImage)
        .count();
    Counts {
        audo: num_audio,
        video: num_video,
        img: num_img,
    }
}
