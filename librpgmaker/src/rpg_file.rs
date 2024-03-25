use crate::{error::Error, Decrypted, Encrypted, EncryptionState};
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

const RPG_HEADER: &[u8] = &[
    0x52, 0x50, 0x47, 0x4D, 0x56, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Represents a decryptable file in an RpgMaker game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileType {
    /// eg. song1.rpgmvo
    Audio,

    /// eg. video1.rpgmvm
    Video,

    /// eg. actor1.rpgmvp
    Image,
}

#[derive(Debug, Clone)]
pub struct RpgFile<State: EncryptionState> {
    pub data: Vec<u8>,
    file_type: FileType,
    orig_path: PathBuf,
    state: PhantomData<State>,
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
    fn from_encrypted_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "rpgmvo" | "ogg_" => FileType::Audio,
            "rpgmvm" | "m4a_" => FileType::Video,
            "rpgmvp" | "png_" => FileType::Image,
            _ => return None,
        };
        Some(ext)
    }

    fn from_decrypted_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "ogg" => FileType::Audio,
            "m4a" => FileType::Video,
            "png" => FileType::Image,
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
    fn to_extension_decrypted(&self) -> &'static str {
        match self {
            FileType::Audio => "ogg",
            FileType::Video => "m4a",
            FileType::Image => "png",
        }
    }

    #[must_use]
    pub fn to_extension_encrypted(&self) -> &'static str {
        match self {
            FileType::Audio => "rpgmvo",
            FileType::Video => "rpgmvm",
            FileType::Image => "rpgmvp",
        }
    }
}

impl RpgFile<Encrypted> {
    pub fn from_encrypted(path: &Path) -> crate::error::Result<Self> {
        let file_type = FileType::from_encrypted_path(path).ok_or(crate::Error::NotEncrypted)?;
        let data = fs::read(path).map_err(|e| crate::Error::IoError {
            err: e,
            file: path.to_path_buf(),
        })?;
        let orig_path = path.to_path_buf();

        Ok(Self {
            data,
            file_type,
            orig_path,
            state: PhantomData,
        })
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
    pub fn decrypt(mut self, key: &[u8]) -> Result<RpgFile<Decrypted>, Error> {
        if self.data.len() <= 32 {
            return Err(Error::FileTooShort(self.orig_path.clone()));
        }

        self.data.drain(0..16); // strip off rpgmaker header
        let (header, _) = self.data.split_at_mut(16); // get a reference to header

        rpg_xor(header, key); // XOR the header with the key

        Ok(RpgFile {
            data: self.data,
            file_type: self.file_type,
            orig_path: self.orig_path,
            state: PhantomData,
        })
    }
}

impl RpgFile<Decrypted> {
    pub fn from_decrypted(path: &Path) -> crate::error::Result<Self> {
        let file_type = FileType::from_decrypted_path(path).ok_or(crate::Error::Encrypted)?;
        let data = fs::read(path).map_err(|err| crate::Error::IoError {
            err,
            file: path.to_path_buf(),
        })?;
        let orig_path = path.to_path_buf();

        Ok(Self {
            data,
            file_type,
            orig_path,
            state: PhantomData,
        })
    }

    pub fn encrypt(mut self, key: &[u8]) -> crate::error::Result<RpgFile<Encrypted>> {
        let (header, _) = self.data.split_at_mut(16); // get a reference to the header
        rpg_xor(header, key); // encrypt header

        let mut tmp_data = Vec::with_capacity(RPG_HEADER.len() + self.data.len()); // pre-allocate space for self.data
        tmp_data.extend_from_slice(RPG_HEADER); // push the rpg header into the new vec
        tmp_data.append(&mut self.data); // append the (now encrypted) data

        Ok(RpgFile {
            data: tmp_data,
            file_type: self.file_type,
            state: PhantomData,
            orig_path: self.orig_path,
        })
    }
}

impl<State: EncryptionState> RpgFile<State> {
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn original_path(&self) -> &Path {
        &self.orig_path
    }

    pub fn decrypted_path(&self) -> PathBuf {
        let mut path = self.orig_path.clone();
        path.set_extension(self.file_type.to_extension_decrypted());
        path
    }

    pub fn encrypted_path(&self) -> PathBuf {
        let mut path = self.orig_path.clone();
        path.set_extension(self.file_type.to_extension_encrypted());
        path
    }
}

#[inline]
pub fn rpg_xor(data: &mut [u8], key: &[u8]) {
    data.iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}
