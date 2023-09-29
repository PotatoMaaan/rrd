use std::{
    fs,
    path::{Path, PathBuf},
};

/// Represents a decryptable file in an RpgMaker game.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpgFileType {
    /// eg. song1.rpgmvo
    RpgAudio,

    /// eg. video1.rpgmvm
    RpgVideo,

    /// eg. actor1.rpgmvp
    RpgImage,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgFile {
    pub data: Vec<u8>,
    pub file_type: RpgFileType,
    pub new_path: PathBuf,
    pub orig_path: PathBuf,
}

impl RpgFileType {
    /// Checks if a given path is an RpgFile (based on extension)
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
    pub fn scan(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        let ext = match ext {
            "rpgmvo" => RpgFileType::RpgAudio,
            "ogg_" => RpgFileType::RpgAudio,
            "rpgmvm" => RpgFileType::RpgVideo,
            "m4a_" => RpgFileType::RpgVideo,
            "rpgmvp" => RpgFileType::RpgImage,
            "png_" => RpgFileType::RpgImage,
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
    /// let file_type = RpgFileType::RpgVideo;
    ///
    /// let ext = file_type.to_extension();
    ///
    /// assert_eq!(ext, "m4a");
    /// ```
    pub fn to_extension(&self) -> String {
        match self {
            RpgFileType::RpgAudio => "ogg",
            RpgFileType::RpgVideo => "m4a",
            RpgFileType::RpgImage => "png",
        }
        .to_string()
    }
}

impl RpgFile {
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_type = RpgFileType::scan(path)?;

        let data = match fs::read(path) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let ext = file_type.to_extension();

        // checked before
        let mut new_path = path.to_path_buf();
        let _ = new_path.set_extension(ext);

        Some(Self {
            data,
            file_type,
            new_path,
            orig_path: path.to_path_buf(),
        })
    }

    #[allow(unused)]
    pub unsafe fn from_parts(data: Vec<u8>, file_type: RpgFileType, orig_path: PathBuf) -> Self {
        let mut new_path = orig_path.clone();
        new_path.set_extension(file_type.to_extension());

        Self {
            data,
            file_type,
            orig_path,
            new_path,
        }
    }

    pub fn decrypt(&self, key: &[u8]) -> Vec<u8> {
        fn xor(data: &[u8], key: &[u8]) -> Vec<u8> {
            let mut result = Vec::with_capacity(data.len());

            for i in 0..data.len() {
                result.push(data[i] ^ key[i % key.len()]);
            }

            result
        }

        let file = &self.data[16..];
        let cyphertext = &file[..16];
        let mut plaintext = xor(cyphertext, key);
        let mut file = file[16..].to_vec();
        plaintext.append(&mut file);
        plaintext
    }
}
