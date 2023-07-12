use clap::{command, Parser};

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// The game directory containing the main executable file
    pub directory: std::path::PathBuf,
    /// Keep the original (encrypted) file next to the decrypted files
    #[arg(short, long)]
    pub keep_original: bool,
    /// The directory where decrypted files are output to relative to the current directory. This automatically keeps the encrypted files in place. If not specified, the files will be alongside the encrypted ones
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
    /// Just scan the directory for decryptable files, list them and then exit
    #[arg(short, long)]
    pub scan: bool,
    /// Don't print individual files being decrypted
    #[arg(short, long)]
    pub quiet: bool,
    /// Print the key (if present) and exit
    #[arg(long)]
    pub key: bool,
    /// Flatten directory structure of the output into a single directory containg all the files (only effective when --output is specified)
    #[arg(short, long)]
    pub flatten_paths: bool,
}
