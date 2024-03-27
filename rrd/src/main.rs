use anyhow::Context;
use clap::Parser;
use librpgmaker::{rpg_file::RpgFile, Game};
use rand::{distributions::Alphanumeric, Rng};
use std::{fs, time::Instant};

mod cli;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    match args.command {
        cli::Command::DecryptGame {
            game_dir,
            output,
            flatten,
            remove,
            no_update_encryption,
        } => {
            let mut game = Game::new(&game_dir).context("Failed to load game")?;
            let key = game.key().to_vec();

            println!("Loaded game, decrypting...");
            let start_time = Instant::now();

            for (i, file) in game.encrypted_files().into_iter().enumerate() {
                let file = file.context("Failed to load file")?;
                let file = file.decrypt(&key).context("Failed to decrypt file")?;

                let output = if let Some(output) = &output {
                    if flatten {
                        let dec_path = file.decrypted_path();

                        let mut file_name = dec_path
                            .file_stem()
                            .expect("File should always have a name")
                            .to_owned();

                        let ext = dec_path.extension();

                        file_name.push("_");
                        file_name.push(rand_string(10));

                        let file_name = if let Some(ext) = ext {
                            file_name.push(".");
                            file_name.push(ext);
                            file_name
                        } else {
                            file_name
                        };

                        output.join(file_name)
                    } else {
                        let dec_path = file.decrypted_path();
                        let new_path = dec_path.strip_prefix(&game_dir).expect(
                            "The decrypted path should always be relative to the base path",
                        );
                        output.join(new_path)
                    }
                } else {
                    file.decrypted_path()
                };

                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent).context("Failed to create parent dir")?;
                }

                println!("[{}] {}", i + 1, output.display());

                fs::write(&output, &file.data).context("Failed to write file")?;

                if remove {
                    fs::remove_file(file.original_path()).context("Failed to delete file")?;
                }
            }

            if !no_update_encryption {
                println!("Updating game encryption state");
                game.set_encrypted_audio(false)
                    .context("Failed to set encrypted audio")?;
                game.set_encrypted_imgs(false)
                    .context("Failed to set encrypted images")?;
            }

            println!("\nDecryption done, took {:.2?}", start_time.elapsed());
        }

        cli::Command::EncryptGame { game_dir } => {}

        cli::Command::Info { game_dir } => {
            let game = Game::new(game_dir).context("Failed to load game")?;
            let title = game.title().unwrap_or("");
            let has_enc_imgs = game.has_encrypted_images();
            let has_enc_audio = game.has_encrypted_audio();

            println!("Found Game: {} ", title);

            println!("\n   Has encrypted audio: {}", has_enc_audio);
            println!("   Has encrypted imgs: {}", has_enc_imgs);
            println!("   Encryption key: {}\n", hex::encode(game.key()));
        }
        cli::Command::Key { game_dir } => {
            let game = Game::new(game_dir).context("Failed to load game")?;
            println!("{}", hex::encode(game.key()));
        }

        cli::Command::EncryptFile { file, key, output } => {}

        cli::Command::DecryptFile { file, key, output } => {
            let file = RpgFile::from_any_path(&file).context("Failed to load file")?;
            let file = match file.is_encrypted() {
                librpgmaker::EncryptionState::Encrypted(e) => e,
                librpgmaker::EncryptionState::Decrypted(_) => {
                    anyhow::bail!("File is not encrypted!");
                }
            };

            let key = hex::decode(key).context("Key was not in valid hex format")?;
            let file = file.decrypt(&key).context("Failed to decrypt file")?;

            let output = output.unwrap_or_else(|| file.decrypted_path());
            println!("Writing to {}", output.display());
            fs::write(&output, file.data).context("Failed to write file")?;
        }

        cli::Command::RestoreImg { img } => todo!(),
    };

    Ok(())
}

fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
