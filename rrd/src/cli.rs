use clap::{command, Parser, Subcommand};
use librpgmaker::OutputSettings;
use std::path::PathBuf;

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// Don't print individual files during decryption
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    DecryptFile {
        path: PathBuf,

        key: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    EncryptFile {
        path: PathBuf,

        key: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    DecryptGame {
        path: PathBuf,

        #[command(subcommand)]
        output: Option<OutputSettings>,
    },

    RestoreFile {
        path: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Scan {
        path: PathBuf,
    },

    Key {
        path: PathBuf,
    },
}
