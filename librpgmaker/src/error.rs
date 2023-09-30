use std::{
    fmt::Display,
    num::ParseIntError,
    path::{PathBuf, StripPrefixError},
};

/// Represents an Error from the library.
#[derive(Debug)]
pub enum Error {
    /// The System.json file was not found.
    /// This probably means that the given
    /// directory is not a valid RpgMaker game.
    SystemJsonNotFound,

    /// Error while interacting with the filesystem.
    IoError(std::io::Error),

    /// The System.json file was not valid JSON.
    /// See the included error for more details.
    SystemJsonInvalidJson(serde_json::Error),

    /// The System.json file dod not contain
    /// the included key.
    SystemJsonKeyNotFound { key: String },

    /// The included key was not in the expected format.
    SystemJsonInvalidKey { key: String },

    /// Stripping a path prefix failed, see error for
    /// more details
    StrixPrefixFailed(StripPrefixError),

    /// Failed to parse a key from System.json
    KeyParseError(ParseIntError),

    /// The given output dir already exists.
    OutputDirExists(PathBuf),

    /// The game is not encrypted.
    NotEncrypted,

    /// The file is to short to be decrypted
    FileToShort(PathBuf),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            Error::SystemJsonNotFound => {
                format!("The system.json file was not found. Make sure the directory is correct.")
            }
            Error::IoError(io_err) => format!("IO Error: {}", io_err),
            Error::SystemJsonInvalidJson(serde_err) => {
                format!("Failed parsing JSON in system.json: {}", serde_err)
            }
            Error::SystemJsonKeyNotFound { key } => {
                format!("The key '{}' was not present in system.json", key)
            }
            Error::SystemJsonInvalidKey { key } => {
                format!("The key '{}' in system.json was an invalid format", key)
            }
            Error::StrixPrefixFailed(err) => format!("{}", err),
            Error::KeyParseError(err) => format!("{}", err),
            Error::OutputDirExists(path) => {
                format!("The output directory '{}' already exists!", path.display())
            }
            Error::NotEncrypted => format!("The game is not encrypted"),
            Error::FileToShort(path) => {
                format!(
                    "The following file was to short to decrypt:\n   -> {}",
                    path.display()
                )
            }
        };

        write!(f, "{}", content)
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::KeyParseError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<StripPrefixError> for Error {
    fn from(value: StripPrefixError) -> Self {
        Self::StrixPrefixFailed(value)
    }
}
