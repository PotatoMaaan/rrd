//! A Library to interact with and decrypt RpgMaker games.
//! To get started, see the `RpgGame` struct.

use crate::system_json::SystemJson;
use rpg_file::RpgFile;
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

pub mod error;
mod rpg_file;
mod system_json;
#[cfg(test)]
mod tests;
pub use error::Error;

pub trait UnknownEncryptionState {}
pub trait EncryptionState {}

#[derive(Debug)]
pub struct Decrypted;
#[derive(Debug)]
pub struct Encrypted;
#[derive(Debug)]
pub struct UnknownEncryption;

#[derive(Debug)]
pub enum Encryption {
    Encrypted(Game<Encrypted>),
    Decrypted(Game<Decrypted>),
}

impl UnknownEncryptionState for Encrypted {}
impl UnknownEncryptionState for Decrypted {}
impl UnknownEncryptionState for UnknownEncryption {}

impl EncryptionState for Encrypted {}
impl EncryptionState for Decrypted {}

#[derive(Debug)]
pub struct Game<ImgState: UnknownEncryptionState> {
    path: PathBuf,
    state: PhantomData<ImgState>,
    system_json: SystemJson,
    key: Vec<u8>,
}

impl<State: UnknownEncryptionState> Game<State> {
    pub fn game_title(&self) -> Option<&str> {
        self.system_json.game_title()
    }

    pub fn key(&self) -> &[u8] {
        &self.key
    }

    pub fn set_encryption_state(&mut self, state: Encryption) -> crate::error::Result<()> {
        self.system_json.set_encryption_state(state)
    }
}

impl Game<UnknownEncryption> {
    pub fn new(path: impl AsRef<Path>) -> Result<Game<UnknownEncryption>, crate::Error> {
        let path = path.as_ref();
        let system_json = SystemJson::find_system_json(path)?;
        let key = system_json.key()?;

        Ok(Game::<UnknownEncryption> {
            path: path.to_path_buf(),
            state: PhantomData,
            system_json,
            key,
        })
    }
}

impl Game<UnknownEncryption> {
    pub fn check_encrypted(self) -> Encryption {
        if self.system_json.is_encrypted() {
            Encryption::Encrypted(Game::<Encrypted> {
                path: self.path,
                state: PhantomData,
                system_json: self.system_json,
                key: self.key,
            })
        } else {
            Encryption::Decrypted(Game::<Decrypted> {
                path: self.path,
                state: PhantomData,
                system_json: self.system_json,
                key: self.key,
            })
        }
    }
}

impl Game<Encrypted> {
    pub fn decrypt(self) -> WalkGameIter<Decrypted> {
        WalkGameIter {
            iter: walkdir::WalkDir::new(&self.path).into_iter(),
            state: PhantomData,
            key: self.key,
        }
    }
}

impl Game<Decrypted> {
    pub fn encrypt(self) -> WalkGameIter<Encrypted> {
        WalkGameIter {
            iter: walkdir::WalkDir::new(&self.path).into_iter(),
            state: PhantomData,
            key: self.key,
        }
    }
}

pub struct WalkGameIter<State: EncryptionState> {
    key: Vec<u8>,
    iter: walkdir::IntoIter,
    state: PhantomData<State>,
}

impl Iterator for WalkGameIter<Decrypted> {
    type Item = Result<RpgFile<Decrypted>, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            match next {
                Ok(next) => {
                    let file = match RpgFile::from_encrypted(next.path()) {
                        Ok(v) => v,
                        Err(_) => {
                            continue;
                        }
                    };
                    let file = match file.decrypt(&self.key) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
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

// impl Iterator for WalkGameIter<Decrypted> {
//     type Item = Result<RpgFile<Decrypted>, crate::Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(next) = self.iter.next() {
//             match next {
//                 Ok(next) => {
//                     todo!()
//                 }
//                 Err(e) => {
//                     return Some(Err(crate::Error::WalkDirError(e)));
//                 }
//             }
//         }

//         None
//     }
// }
