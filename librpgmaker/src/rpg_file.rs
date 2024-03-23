use crate::error::Error;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents a decryptable file in an RpgMaker game.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileType {
    /// eg. song1.rpgmvo
    Audio,

    /// eg. video1.rpgmvm
    Video,

    /// eg. actor1.rpgmvp
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgFile {
    pub data: Vec<u8>,
    pub file_type: FileType,
    orig_path: PathBuf,
}

impl FileType {
    /// Checks if a given path is an `RpgFile` (based on extension)
    ///
    /// ## Example
    /// ```
    /// use std::path::Path;
    /// use librpgmaker::prelude::*;
    ///
    /// let path = Path::new("test/actor1.rpgmvp");
    ///
    /// let is_rpgfile = RpgFileType::scan(&path);
    ///
    /// assert!(is_rpgfile.is_some());
    /// ```
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "rpgmvo" | "ogg_" => FileType::Audio,
            "rpgmvm" | "m4a_" => FileType::Video,
            "rpgmvp" | "png_" => FileType::Image,
            _ => return None,
        };
        Some(ext)
    }

    /// Returns a "decrypted" file extension
    ///
    /// ## Example
    /// ```
    /// use librpgmaker::prelude::*;
    ///
    /// let file_type = RpgFileType::Video;
    ///
    /// let ext = file_type.to_extension();
    ///
    /// assert_eq!(ext, "m4a");
    /// ```
    #[must_use]
    pub fn to_extension_decrypted(&self) -> String {
        match self {
            FileType::Audio => "ogg",
            FileType::Video => "m4a",
            FileType::Image => "png",
        }
        .to_string()
    }

    #[must_use]
    pub fn to_extension_encrypted(&self) -> String {
        todo!()
    }
}

impl RpgFile {
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_type = FileType::from_path(path)?;

        let Ok(data) = fs::read(path) else {
            return None;
        };

        Some(Self {
            data,
            file_type,
            orig_path: path.to_path_buf(),
        })
    }

    #[allow(unused)]
    pub unsafe fn from_raw_parts(data: Vec<u8>, file_type: FileType, orig_path: PathBuf) -> Self {
        let mut new_path = orig_path.clone();
        new_path.set_extension(file_type.to_extension_decrypted());

        Self {
            data,
            file_type,
            orig_path,
        }
    }

    /// Decrypts the data in the file.
    ///
    /// File before decryption:
    ///
    /// | *RPGmaker header (16 bytes)* | *encrypted header (16 bytes)* | *rest of the data* |
    ///
    /// to undo to this, we just need to discard the first 16 bytes,
    /// xor the encrypted header with the key and stick the data
    /// underneith the decrypted header.
    ///
    /// File after decryption:
    ///
    /// | *header (16 bytes)* | *rest of the data* |
    pub fn decrypt(&mut self, key: &[u8]) -> Result<(), Error> {
        if self.data.len() <= 32 {
            return Err(Error::FileTooShort(self.orig_path.clone()));
        }

        self.data.drain(0..16); // strip off rpgmaker header
        let (header, _) = self.data.split_at_mut(16); // get a reference to header
        header
            .iter_mut()
            .enumerate()
            .for_each(|(i, d)| *d ^= key[i % key.len()]); // XOR the header with the key
        Ok(())
    }
}
