use crate::librpgmaker::OutputSettings;
use clap::{command, Parser, Subcommand};
use std::path::PathBuf;

/// Decrypt files encryped by RPMVs default encryprion{n}{n}
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Decrypt {
        game_dir: PathBuf,

        #[command(subcommand)]
        output: Option<OutputSettings>,
    },
    Scan {
        game_dir: PathBuf,
    },

    /// Just prints the key
    Key {
        game_dir: PathBuf,
    },
}
