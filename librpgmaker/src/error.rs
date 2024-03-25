use std::{fmt::Display, num::ParseIntError, path::PathBuf};

/// Represents an Error from the library.
#[derive(Debug)]
pub enum Error {
    /// The System.json file was not found.
    /// This probably means that the given
    /// directory is not a valid RpgMaker game.
    SystemJsonNotFound,

    /// Error while interacting with the filesystem.
    IoError { err: std::io::Error, file: PathBuf },

    /// Error while walking directory tree
    WalkDirError(walkdir::Error),

    /// The System.json file was not valid JSON.
    /// See the included error for more details.
    SystemJsonInvalidJson(serde_json::Error),

    /// The System.json file dod not contain
    /// the included key.
    SystemJsonKeyNotFound { key: String },

    /// The included key was not in the expected format.
    SystemJsonInvalidKey { key: String },

    /// Failed to parse a key from System.json
    KeyParseError(ParseIntError),

    /// The given output dir already exists.
    OutputDirExists(PathBuf),

    /// The game is not encrypted (but should be).
    NotEncrypted,

    /// The game is encrypted (but should not be).
    Encrypted,

    /// The file is to short to be decrypted
    FileTooShort(PathBuf),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Error::SystemJsonNotFound => {
                format!("The system.json file was not found. Make sure the directory is correct.")
            }
            Error::IoError { err, file } => format!("IO Error on file {}: {}", file.display(), err),
            Error::SystemJsonInvalidJson(serde_err) => {
                format!("Failed parsing JSON in system.json: {}", serde_err)
            }
            Error::SystemJsonKeyNotFound { key } => {
                format!("The key '{}' was not present in system.json", key)
            }
            Error::SystemJsonInvalidKey { key } => {
                format!("The key '{}' in system.json was an invalid format", key)
            }
            Error::KeyParseError(err) => format!("{}", err),
            Error::OutputDirExists(path) => {
                format!("The output directory '{}' already exists!", path.display())
            }
            Error::NotEncrypted => format!("The game is not encrypted (even though it should be)"),
            Error::FileTooShort(path) => {
                format!(
                    "The following file was too short to decrypt:\n   -> {}",
                    path.display()
                )
            }
            Error::WalkDirError(e) => format!("Error while walking directory: {}", e),
            Error::Encrypted => format!("The game is encrypted (even though it should not be)"),
        };

        write!(f, "{}", content)
    }
}

impl std::error::Error for Error {}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::KeyParseError(value)
    }
}

pub type Result<T> = std::result::Result<T, crate::Error>;
