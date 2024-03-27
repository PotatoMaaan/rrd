//! A Library to interact with and decrypt RpgMaker games.
//! To get started, see the `RpgGame` struct.

use crate::system_json::SystemJson;
use rpg_file::RpgFile;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub mod error;
pub mod rpg_file;
mod system_json;
#[cfg(test)]
mod tests;
pub use error::Error;

#[derive(Debug)]
pub struct Encrypted;
#[derive(Debug)]
pub struct Decrypted;
#[derive(Debug)]
pub struct UnknownEncryption;

#[derive(Debug)]
pub enum EncryptionState<E, D> {
    Encrypted(E),
    Decrypted(D),
}

#[derive(Debug)]
pub struct Game {
    path: PathBuf,
    system_json: SystemJson,
    key: Vec<u8>,
}

impl Game {
    /// Returns the title of the game (if available)
    pub fn title(&self) -> Option<&str> {
        self.system_json.game_title()
    }

    /// Returns the encryption key of the game
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    /// Attenpt to create a Game from the given path
    pub fn new(path: impl AsRef<Path>) -> Result<Game, crate::Error> {
        let path = path.as_ref();
        let system_json = SystemJson::find_system_json(path)?;
        let key = system_json.key()?;

        Ok(Game {
            path: path.to_path_buf(),
            system_json,
            key,
        })
    }

    /// Returns an iterator over all decryptable/encryptable files in the game
    pub fn files(&self) -> WalkGameIter<UnknownEncryption> {
        WalkGameIter {
            iter: jwalk::WalkDir::new(self.path.clone()).into_iter(),
            state: PhantomData,
        }
    }

    /// Returns an iterator over all encrypted files in the game
    pub fn encrypted_files(&self) -> WalkGameIter<Encrypted> {
        WalkGameIter {
            iter: jwalk::WalkDir::new(self.path.clone()).into_iter(),
            state: PhantomData,
        }
    }

    /// Returns an iterator over all decrypted files in the game
    pub fn decrypted_files(&self) -> WalkGameIter<Decrypted> {
        WalkGameIter {
            iter: jwalk::WalkDir::new(self.path.clone()).into_iter(),
            state: PhantomData,
        }
    }

    /// Reads information in System.json to determine if the game reports as being encrypted
    pub fn has_encrypted_images(&self) -> bool {
        self.system_json.has_encrypted_images()
    }

    pub fn has_encrypted_audio(&self) -> bool {
        self.system_json.has_encrypted_audio()
    }

    pub fn set_encrypted_audio(&mut self, state: bool) -> crate::error::Result<()> {
        self.system_json.set_encrypted_audio(state)
    }

    pub fn set_encrypted_imgs(&mut self, state: bool) -> crate::error::Result<()> {
        self.system_json.set_encrypted_imgs(state)
    }
}

/// An iterator over files in the game
pub struct WalkGameIter<Enc> {
    iter: jwalk::DirEntryIter<((), ())>,
    state: PhantomData<Enc>,
}

impl Iterator for WalkGameIter<UnknownEncryption> {
    type Item = Result<RpgFile<UnknownEncryption>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            match next {
                Ok(next) => {
                    let file = match RpgFile::from_any_path(&next.path()) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };

                    return Some(Ok(file));
                }
                Err(e) => {
                    return Some(Err(crate::Error::WalkDirError(e)));
                }
            }
        }

        None
    }
}

impl Iterator for WalkGameIter<Encrypted> {
    type Item = Result<RpgFile<Encrypted>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            match next {
                Ok(next) => {
                    let file = match RpgFile::from_encrypted_path(&next.path()) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };

                    return Some(Ok(file));
                }
                Err(e) => {
                    return Some(Err(crate::Error::WalkDirError(e)));
                }
            }
        }

        None
    }
}

impl Iterator for WalkGameIter<Decrypted> {
    type Item = Result<RpgFile<Decrypted>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            match next {
                Ok(next) => {
                    let file = match RpgFile::from_decrypted_path(&next.path()) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };

                    return Some(Ok(file));
                }
                Err(e) => {
                    return Some(Err(crate::Error::WalkDirError(e)));
                }
            }
        }

        None
    }
}
