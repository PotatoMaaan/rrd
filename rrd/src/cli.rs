use clap::{builder::ArgPredicate, command, Parser, Subcommand};
use std::path::PathBuf;

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Decrypt an entire game
    DecryptGame {
        /// The path to the game
        game_dir: PathBuf,

        /// A directory where decrypted files will be stored
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Flattens all files into a single directory
        #[arg(short, long, requires = "output")]
        flatten: bool,

        /// Removes the original encrypted files
        #[arg(short, long, conflicts_with_all = ["output", "flatten"])]
        remove: bool,

        /// Don't tell the game that it's assets are decrypted (the game will continue to use the encrypted assets)
        #[arg(long, conflicts_with_all = ["remove"], default_value_if("output", ArgPredicate::IsPresent, "true"))]
        no_update_encryption: bool,
    },

    /// Encrypt an entire game
    EncryptGame { game_dir: PathBuf },

    /// Print information about a game
    Info { game_dir: PathBuf },

    /// Print the key of the game
    Key { game_dir: PathBuf },

    /// Encrypt a single file with a key
    EncryptFile {
        file: PathBuf,
        key: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Decrypt a single file with a key
    DecryptFile {
        file: PathBuf,
        key: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// "Decrypt" a single image without a key (by rebuilding it's header)
    RestoreImg { img: PathBuf },
}
