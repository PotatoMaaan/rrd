use clap::{command, Parser};
use librpgmaker::OutputSettings;
use std::path::PathBuf;

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// The game directory
    pub game_dir: PathBuf,

    #[command(subcommand)]
    pub output: Option<OutputSettings>,

    /// Don't print individual files during decryption
    #[arg(short, long)]
    pub quiet: bool,

    /// Just scan the amount of decryptable files
    #[arg(short, long)]
    pub scan: bool,

    /// Just print the key
    #[arg(short, long)]
    pub key: bool,
}
