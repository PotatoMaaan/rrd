use clap::{command, Parser, Subcommand};
use librpgmaker::OutputSettings;
use std::path::PathBuf;

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// Don't print individual files during processing
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Decrypt a single file with a key
    DecryptFile {
        /// The path to the file
        path: PathBuf,

        /// The encryption key to use
        key: String,

        /// The output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Encrypt a single file with a key
    EncryptFile {
        /// The path to the file
        path: PathBuf,

        /// The encryption key to use
        key: String,

        /// The output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Restore a single *IMAGE* file (decrypt without a key, might not work 100%)
    RestoreFile {
        /// The path to the file
        path: PathBuf,

        /// The output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Decrypt an entire game
    DecryptGame {
        /// The path to the game
        path: PathBuf,

        /// The output settings (see --help for more info)
        #[command(subcommand)]
        output: Option<OutputSettings>,
    },

    /// Encrypt an entire game
    EncryptGame {
        /// The path to the game
        path: PathBuf,
    },

    /// Scan a game for files
    Scan {
        /// The path to the game
        path: PathBuf,
    },

    /// Print the encryption key of a game
    Key {
        /// The path to the game
        path: PathBuf,
    },
}
