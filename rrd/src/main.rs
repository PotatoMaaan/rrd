use rayon::prelude::*;

mod cli;

fn main() {
    let c = librpgmaker::Game::new("test_files/test_game");

    let c = c.assert_encrypted();

    dbg!(&c);
}
