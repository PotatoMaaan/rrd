//! A Library to interact with and decrypt RpgMaker games.
//! To get started, see the `RpgGame` struct.

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];
const HAS_ENC_AUIDO_KEY: &str = "hasEncryptedAudio";
const HAS_ENC_IMG_KEY: &str = "hasEncryptedImages";
const ENCKEY_KEY: &str = "encryptionKey";

pub mod error;
mod rpg_file;
mod system_json;
#[cfg(test)]
mod tests;

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub use error::Error;
use rpg_file::RpgFile;

pub trait EncryptionState {}

#[derive(Debug)]
pub struct Decrypted;
#[derive(Debug)]
pub struct Encrypted;
#[derive(Debug)]
pub struct UnknownEncryption;

impl EncryptionState for Encrypted {}
impl EncryptionState for Decrypted {}
impl EncryptionState for UnknownEncryption {}

#[derive(Debug)]
pub struct Game<ImgState: EncryptionState> {
    path: PathBuf,
    state: PhantomData<ImgState>,
}

impl Game<UnknownEncryption> {
    pub fn new(path: impl AsRef<Path>) -> Game<UnknownEncryption> {
        Game::<UnknownEncryption> {
            path: path.as_ref().to_path_buf(),
            state: PhantomData,
        }
    }
}

impl Game<UnknownEncryption> {
    pub fn assert_encrypted(self) -> Game<Encrypted> {
        todo!()
    }

    pub fn assert_decrypted(self) -> Game<Decrypted> {
        todo!()
    }
}

impl Game<Encrypted> {
    pub fn decrypt(&self) -> WalkGameIter {
        WalkGameIter {
            iter: Arc::new(Mutex::new(Box::new(
                walkdir::WalkDir::new(&self.path).into_iter(),
            ))),
        }
    }
}

impl Game<Decrypted> {
    pub fn encrypt(&self) -> WalkGameIter {
        WalkGameIter {
            iter: Arc::new(Mutex::new(Box::new(
                walkdir::WalkDir::new(&self.path).into_iter(),
            ))),
        }
    }
}

pub struct WalkGameIter {
    iter: Arc<Mutex<Box<dyn Iterator<Item = Result<walkdir::DirEntry, walkdir::Error>> + Send>>>,
}

impl Iterator for WalkGameIter {
    type Item = Result<RpgFile, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = &mut *self.iter.lock().expect("Mutex poisoned");

        while let Some(next) = iter.next() {
            match next {
                Ok(next) => {
                    if let Some(next) = RpgFile::from_path(next.path()) {
                        return Some(Ok(next));
                    }
                }
                Err(e) => {
                    return Some(Err(crate::Error::WalkDirError(e)));
                }
            }
        }

        None
    }
}
