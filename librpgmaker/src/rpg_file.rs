use crate::{error::Error, rpg_xor};
use std::{
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

/// The fake header inserted into """encrypted""" RpgMaker files
const RPG_HEADER: &[u8] = &[
    0x52, 0x50, 0x47, 0x4D, 0x56, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// A valid PNG header to be inserted into files during the restore process
const PNG_HEADER: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0xD, 0xA, 0x1A, 0xA, 0x0, 0x0, 0x0, 0xD, 0x49, 0x48, 0x44, 0x52,
];

/// Represents a decryptable file in an RpgMaker game.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpgFileType {
    /// eg. audio1.rpgmvo
    Audio,

    /// eg. video1.rpgmvm
    Video,

    /// eg. actor1.rpgmvp
    Image,
}

pub struct Encrypted;
pub struct Decrypted;

/// Contains a valid RpgMaker file.
///
/// Contains functions to decrypt / encrypt / resote the file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgFile<State> {
    data: Vec<u8>,
    file_type: RpgFileType,
    decrypted_path: PathBuf,
    encrypted_path: PathBuf,
    state: PhantomData<State>,
}

impl RpgFileType {
    /// Checks if `path` is a valid RpgFileType **based on the extension alone**!
    ///
    /// Assumes that the file is **encrypted**
    ///
    /// ## Example
    /// ```
    /// use std::path::Path;
    /// use librpgmaker::prelude::*;
    ///
    /// let path = Path::new("test/actor1.rpgmvp");
    ///
    /// let is_rpgfile = RpgFileType::from_encrypted_path(&path);
    ///
    /// assert!(is_rpgfile.is_some());
    /// ```
    #[must_use]
    pub fn from_encrypted_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "rpgmvo" | "ogg_" => Self::Audio,
            "rpgmvm" | "m4a_" => Self::Video,
            "rpgmvp" | "png_" => Self::Image,
            _ => return None,
        };
        Some(ext)
    }

    /// Checks if `path` is a valid RpgFileType **based on the extension alone**!
    ///
    /// Assumes that the file is **decrypted**
    ///
    /// ## Example
    /// ```
    /// use std::path::Path;
    /// use librpgmaker::prelude::*;
    ///
    /// let path = Path::new("test/actor1.png");
    ///
    /// let is_rpgfile = RpgFileType::from_decrypted_path(&path);
    ///
    /// assert!(is_rpgfile.is_some());
    /// ```
    pub fn from_decrypted_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "png" => Self::Image,
            "m4a" => Self::Video,
            "ogg" => Self::Audio,
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
    /// let ext = file_type.to_extension_decrypted();
    ///
    /// assert_eq!(ext, "m4a");
    /// ```
    pub fn to_extension_decrypted(&self) -> String {
        match self {
            Self::Audio => "ogg",
            Self::Video => "m4a",
            Self::Image => "png",
        }
        .to_string()
    }

    /// Returns an "encrypted" file extension
    ///
    /// ## Example
    /// ```
    /// use librpgmaker::prelude::*;
    ///
    /// let file_type = RpgFileType::Video;
    ///
    /// let ext = file_type.to_extension_encrypted();
    ///
    /// assert_eq!(ext, "rpgmvm");
    /// ```
    pub fn to_extension_encrypted(&self) -> String {
        match self {
            Self::Audio => "rpgmvo",
            Self::Video => "rpgmvm",
            Self::Image => "rpgmvp",
        }
        .to_string()
    }

    /// Returns an "underscored" file extension
    ///
    /// ### Note:
    /// I have no idea why this is even a thing
    /// but some games use this as an encrypted extension
    /// so it's here for the sake of completeness
    ///
    /// ## Example
    /// ```
    /// use librpgmaker::prelude::*;
    ///
    /// let file_type = RpgFileType::Video;
    ///
    /// let ext = file_type.to_extension_underscored();
    ///
    /// assert_eq!(ext, "m4a_");
    /// ```
    pub fn to_extension_underscored(&self) -> String {
        match self {
            Self::Audio => "ogg_",
            Self::Video => "m4a_",
            Self::Image => "png_",
        }
        .to_string()
    }
}

impl RpgFile<Encrypted> {
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
    pub fn decrypt(mut self, key: &[u8]) -> Result<RpgFile<Decrypted>, Error> {
        if self.data.len() <= 32 {
            return Err(Error::FileTooShort(self.encrypted_path.clone()));
        }

        self.data.drain(0..16); // strip off rpgmaker header
        let (header, _) = self.data.split_at_mut(16); // get a reference to header

        rpg_xor(header, key); // XOR the header with the key

        Ok(RpgFile {
            data: self.data,
            file_type: self.file_type,
            decrypted_path: self.decrypted_path,
            encrypted_path: self.encrypted_path,
            state: PhantomData::<Decrypted>,
        })
    }

    /// Restores **IMAGE** files without a key
    ///
    /// # Panics
    /// if the file is not RpgFileType::Image!
    ///
    /// The reason why it only restores images is that only the png
    /// files have a static first 16 bytes as the header.
    /// Other files have unique elements on the first 16 bytes of header,
    /// which is the part that unavailable without a key.
    pub fn restore_img(mut self) -> Result<RpgFile<Decrypted>, Error> {
        if self.file_type != RpgFileType::Image {
            panic!("Tried to restore non-image RpgFile!");
        }

        if self.data.len() <= 32 {
            return Err(Error::FileTooShort(self.encrypted_path.clone()));
        }

        self.data.drain(0..16); // strip off rpgmaker header
        self.data.splice(0..16, PNG_HEADER.to_vec().into_iter()); // splice in the known PNG header

        Ok(RpgFile {
            data: self.data,
            file_type: self.file_type,
            decrypted_path: self.decrypted_path,
            encrypted_path: self.encrypted_path,
            state: PhantomData::<Decrypted>,
        })
    }

    /// Tries to create an RpgFile from an **encrypted** file.
    pub fn from_path_encrypted<P: AsRef<Path>>(path: P) -> Option<Self> {
        let file_type = RpgFileType::from_encrypted_path(path.as_ref())?;

        let data = fs::read(&path).ok()?;
        let ext = file_type.to_extension_decrypted();

        let mut new_path = path.as_ref().to_path_buf();
        let _ = new_path.set_extension(ext);

        Some(Self {
            data,
            file_type,
            decrypted_path: new_path,
            encrypted_path: path.as_ref().to_path_buf(),
            state: PhantomData::<Encrypted>,
        })
    }

    #[allow(unused)]
    pub unsafe fn from_raw_parts_encrypted(
        data: Vec<u8>,
        file_type: RpgFileType,
        orig_path: PathBuf,
    ) -> Self {
        let mut new_path = orig_path.clone();
        new_path.set_extension(file_type.to_extension_decrypted());

        RpgFile {
            data,
            file_type,
            decrypted_path: new_path,
            encrypted_path: orig_path,
            state: PhantomData::<Encrypted>,
        }
    }
}

impl RpgFile<Decrypted> {
    /// Encrypts the contents of the current file.
    ///
    /// Works like the `decrypt()` function but in reverse
    ///
    /// # Performance
    /// slightly slower than decryption, since we *have* to clone into a new vector
    /// since we can't push to the front of a vector.
    pub fn encrypt(mut self, key: &[u8]) -> Result<RpgFile<Encrypted>, Error> {
        let (header, _) = self.data.split_at_mut(16); // get a reference to the header
        rpg_xor(header, key); // encrypt header

        let mut tmp_data = Vec::with_capacity(RPG_HEADER.len() + self.data.len()); // pre-allocate space for self.data
        tmp_data.extend_from_slice(RPG_HEADER); // push the rpg header into the new vec
        tmp_data.append(&mut self.data); // append the (now encrypted) data

        Ok(RpgFile {
            data: tmp_data,
            file_type: self.file_type,
            decrypted_path: self.decrypted_path,
            encrypted_path: self.encrypted_path,
            state: PhantomData::<Encrypted>,
        })
    }

    /// Constructs an `RpgFile` from the given parts. It is the callers responsibity to ensure that
    /// the given data matches.
    #[allow(unused)]
    pub unsafe fn from_raw_parts_decrypted(
        data: Vec<u8>,
        file_type: RpgFileType,
        orig_path: PathBuf,
    ) -> Self {
        let mut new_path = orig_path.clone();
        new_path.set_extension(file_type.to_extension_decrypted());

        RpgFile {
            data,
            file_type,
            decrypted_path: new_path,
            encrypted_path: orig_path,
            state: PhantomData::<Decrypted>,
        }
    }

    /// Tries to create an RpgFile from a **decrypted** file.
    pub fn from_path_decrypted<P: AsRef<Path>>(path: P) -> Option<Self> {
        let file_type = RpgFileType::from_decrypted_path(path.as_ref())?;

        let data = fs::read(&path).ok()?;
        let ext = file_type.to_extension_encrypted();

        let mut new_path = path.as_ref().to_path_buf();
        let _ = new_path.set_extension(ext);

        Some(Self {
            data,
            file_type,
            decrypted_path: path.as_ref().to_path_buf(),
            encrypted_path: new_path,
            state: PhantomData::<Decrypted>,
        })
    }
}

impl<State> RpgFile<State> {
    /// Returns a reference to the encrypted path
    #[inline]
    pub fn encrypted_path(&self) -> &Path {
        &self.encrypted_path
    }

    /// Returns a reference to the decrypted path
    #[inline]
    pub fn decrypted_path(&self) -> &Path {
        &self.decrypted_path
    }

    /// Returns a view into the data of the file
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
