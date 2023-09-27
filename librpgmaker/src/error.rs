use std::{
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
