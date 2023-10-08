use clap::Parser;
use cli::*;
use itertools::Itertools;
use librpgmaker::prelude::*;
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::exit,
    time::{Duration, Instant},
};

mod cli;

const DEFAULT_OUTPUT: OutputSettings = OutputSettings::NextTo;

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::DecryptFile { path, output, key } => {
            let Some(file) = RpgFile::from_path_encrypted(&path) else {
                eprintln!("File is not a valid RpgMaker file!");
                exit(1);
            };

            let Ok(key_bytes) = librpgmaker::decode_hex(&key) else {
                eprintln!("Invalid key!");
                exit(1);
            };

            let file = match file.decrypt(&key_bytes) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed decrypting file: {}", e);
                    exit(1);
                }
            };

            let out_path = output.unwrap_or(file.decrypted_path().to_path_buf());

            if let Err(err) = fs::write(&out_path, file.data()) {
                eprintln!("Failed writing to {}: {}", out_path.display(), err);
            }

            println!("Decrypted '{}' with key '{}'", path.display(), key,);
            println!("Saved to: {}", out_path.display());
        }

        Commands::EncryptFile {
            ref path,
            key,
            output,
        } => {
            let Some(file) = RpgFile::from_path_decrypted(&path) else {
                eprintln!(
                    "'{}' cannot be encrypted with RPGmaker encryption.",
                    &path.display()
                );

                exit(1);
            };

            let Ok(key_bytes) = librpgmaker::decode_hex(&key) else {
                eprintln!("Invalid key!");
                exit(1);
            };

            let file = match file.encrypt(&key_bytes) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed encrypting file: {}", e);
                    exit(1);
                }
            };

            let out_path = output.unwrap_or(file.encrypted_path().to_path_buf());

            if let Err(e) = fs::write(&out_path, file.data()) {
                eprintln!("Failed writing to file: {}", e);
                exit(1);
            };

            println!("Decrypted '{}' with key '{}'", path.display(), key);
            println!("Saved to: {}", out_path.display());
        }

        Commands::RestoreFile { ref path, output } => {
            let Some(file) = RpgFile::from_path_encrypted(path) else {
                eprintln!("{} is not a valid RpgMaker file!", path.display());
                exit(1);
            };

            let file = match file.restore_img() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to restore file: {}", e);
                    exit(1);
                }
            };

            let out_path = output.unwrap_or(file.decrypted_path().to_path_buf());
            fs::write(&out_path, file.data()).unwrap();

            println!("Restored '{}'", path.display());
            println!("Saved to: {}", out_path.display());
        }

        Commands::Scan { ref path } => {
            let game = get_game(&path, &args);

            let encrypted = game.scan_encrypted_files().unwrap_or_else(|e| {
                eprintln!("Failed to scan game: {}", e);
                exit(1);
            });
            let decrypted = game.scan_decrypted_files().unwrap_or_else(|e| {
                eprintln!("Failed to scan game: {}", e);
                exit(1);
            });

            let count_enc = count_variants(encrypted.iter());
            let count_dec = count_variants(decrypted.iter());

            pretty_print_key(&game);
            println!(
                "Found {} encrypted files:\n{}",
                count_enc.total(),
                count_enc
            );
            println!(
                "Found {} decrypted files:\n{}",
                count_dec.total(),
                count_dec
            );
        }

        Commands::Key { ref path } => {
            let game = get_game(path, &args);

            pretty_print_key(&game);
        }

        Commands::DecryptGame {
            ref path,
            ref output,
        } => {
            let mut game = get_game(&path, &args);

            let start_time = Instant::now();
            let results = game
                .decrypt_all(&output.clone().unwrap_or(DEFAULT_OUTPUT))
                .unwrap_or_else(|e| {
                    eprintln!("Failed decrypting game: {}", e);
                    exit(1);
                });

            let elapsed = start_time.elapsed();
            let results_len = results.len();
            let (succeeded, failed) = split_results(results);

            if !failed.is_empty() {
                println!();

                for err in &failed {
                    println!("ERROR: {}", err);
                }

                println!("{} errors encrountered during encryption.", failed.len());
            }

            if !succeeded.is_empty() {
                let avg = avg_durations(&succeeded);
                let (max_dir, max_path) =
                    succeeded.iter().max_by(|(a, _), (b, _)| a.cmp(b)).unwrap();

                println!(
                    "\n\nDecrypted {}/{} files in {:.2?}",
                    succeeded.len(),
                    results_len,
                    elapsed
                );

                println!("\nAverage time per file: {:.2?}", avg);
                println!(
                    "\nMax time taken: {:.2?} by '{}'",
                    max_dir,
                    max_path.display()
                );
            } else {
                println!("Decryption failed. See errors above.");
            }
        }

        Commands::EncryptGame { ref path } => {
            let mut game = get_game(path, &args);
            let (succ, fail) = split_results(game.encrypt_all().unwrap_or_else(|e| {
                eprintln!("Failed to encrypt files: {}", e);
                exit(1);
            }));

            println!("\n\nEncrypted {} files.", succ.len());
            println!("Failed encrypting {} files.", fail.len());
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

impl Counts {
    fn total(&self) -> usize {
        self.audio + self.video + self.audio
    }
}

impl Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "   - images: {}\n   - audios: {}\n   - videos: {}\n",
            self.image, self.audio, self.video
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
