use std::{fs, path::Path};

use itertools::Itertools;
use librpgmaker::Encryption;
use rayon::prelude::*;

mod cli;

fn main() {
    let base_path = Path::new("test_files/test_game");
    let game = librpgmaker::Game::new(base_path).expect("game not found");

    let game = match game.check_encrypted() {
        Encryption::Encrypted(game) => game,
        Encryption::Decrypted(game) => panic!("Game was not encrypted"),
    };

    println!(
        "Found game title: {}",
        game.game_title().unwrap_or("[NO TITLE FOUND]")
    );
    println!("Found key: {:x?}", game.key());

    for file in game.decrypt() {
        let file = file.unwrap();
        let path = file.decrypted_path();
        let path = path.strip_prefix(base_path).unwrap();
        let path = Path::new("test_files/output").join(path);
        if let Some(base) = path.parent() {
            fs::create_dir_all(base).unwrap();
        }
        println!("{}", path.display());
        fs::write(path, file.data).unwrap();
    }
}
