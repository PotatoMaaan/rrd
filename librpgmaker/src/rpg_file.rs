use crate::error::Error;
use std::{
    fs,
    path::{Path, PathBuf},
};

const RPG_HEADER: &[u8] = &[
    0x52, 0x50, 0x47, 0x4D, 0x56, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Represents a decryptable file in an RpgMaker game.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpgFileType {
    /// eg. song1.rpgmvo
    Audio,

    /// eg. video1.rpgmvm
    Video,

    /// eg. actor1.rpgmvp
    Image,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgFile {
    data: Vec<u8>,
    file_type: RpgFileType,
    new_path: PathBuf,
    orig_path: PathBuf,
}

impl RpgFileType {
    pub fn from_unencrypted<P: AsRef<Path>>(path: P) -> Option<Self> {
        todo!("implement!!!");
    }

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
    pub fn scan(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "rpgmvo" | "ogg_" => RpgFileType::Audio,
            "rpgmvm" | "m4a_" => RpgFileType::Video,
            "rpgmvp" | "png_" => RpgFileType::Image,
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
    pub fn to_extension(&self) -> String {
        match self {
            RpgFileType::Audio => "ogg",
            RpgFileType::Video => "m4a",
            RpgFileType::Image => "png",
        }
        .to_string()
    }
}

impl RpgFile {
    /// Attempts to construct Self from a given path
    ///
    /// Returns Some(`RpgFile`) when the file is an `RpgFile` and
    /// None if it's not
    /// > Note that this is based on filename alone.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let file_type = RpgFileType::scan(path.as_ref())?;

        let Ok(data) = fs::read(&path) else {
            return None;
        };

        let ext = file_type.to_extension();

        let mut new_path = path.as_ref().to_path_buf();
        let _ = new_path.set_extension(ext);

        Some(Self {
            data,
            file_type,
            new_path,
            orig_path: path.as_ref().to_path_buf(),
        })
    }

    /// Retuns the orignal path of the `RpgFile`.
    ///
    /// eg. `test_files/www/img/test.rpgmvp`
    #[inline]
    pub fn orig_path(&self) -> &Path {
        &self.orig_path
    }

    /// Returns the new path of the `RpgFile`
    ///
    /// eg. `test_files/www/img//test.png`
    #[inline]
    pub fn new_path(&self) -> &Path {
        &self.new_path
    }

    /// Provides a view into the files data
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Constructs an `RpgFile` from the given parts. It is the callers responsibity to ensure that
    /// the given data matches.
    #[allow(unused)]
    pub unsafe fn from_parts(data: Vec<u8>, file_type: RpgFileType, orig_path: PathBuf) -> Self {
        let mut new_path = orig_path.clone();
        new_path.set_extension(file_type.to_extension());

        Self {
            data,
            file_type,
            new_path,
            orig_path,
        }
    }

    /// Decrypts the data in the file.
    ///
    /// # Errors
    /// If the file is too short to be decrypted, an error is returned.
    ///
    /// # Explaining the decryption
    /// File before decryption:
    ///
    /// `| RPGmaker header (16 bytes) | encrypted header (16 bytes) | rest of the data |`
    ///
    /// To undo this encryption, we just need to strip off the rpgmaker header (first 16 bytes)
    /// and then xor the header with the key. We don't even need to touch the actual data,
    /// since it's not encrypted.
    ///
    /// File after decryption:
    ///
    /// `| header (16 bytes) | rest of the data |`
    ///
    /// # Performance
    /// The actual decryption is O(1), but the removing of the first 16 bytes of the data is O(n)
    /// worst case, where n in the length of the data. This is because the vector needs to be
    /// shifted back, which can only be done by copying.
    ///
    /// Overall this operarion is O(n) + 1 where n is the length of the data and one is the
    /// actual decryption of the header (fixed size). 99% of the performance impact will be from
    /// copying the elements back (the call to `drain()`).
    pub fn decrypt(&mut self, key: &[u8]) -> Result<(), Error> {
        if self.data.len() <= 32 {
            return Err(Error::FileTooShort(self.orig_path.clone()));
        }

        self.data.drain(0..16); // strip off rpgmaker header
        let (header, _) = self.data.split_at_mut(16); // get a reference to header

        Self::rpg_xor(header, key); // XOR the header with the key

        Ok(())
    }

    fn rpg_xor(data: &mut [u8], key: &[u8]) {
        data.iter_mut()
            .enumerate()
            .for_each(|(i, d)| *d ^= key[i % key.len()])
    }

    pub fn encrypt(&mut self, key: &[u8]) -> Result<(), Error> {
        let (header, _) = self.data.split_at_mut(16); // get a reference to the header
        Self::rpg_xor(header, key); // encrypt header

        let mut tmp_data = Vec::with_capacity(RPG_HEADER.len() + self.data.len()); // pre-allocate space for self.data
        tmp_data.extend_from_slice(RPG_HEADER); // push the rpg header into the new vec
        tmp_data.append(&mut self.data); // append the (now encrypted) data

        self.data = tmp_data; // assign self.data to the new data
        Ok(())
    }
}
