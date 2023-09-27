//! A Library to interact with and decrypt RpgMaker games.
//! To get started, see the `RpgGame` struct.

use error::Error;
use rpg_file::{RpgFile, RpgFileType};
use serde_json::Value;
use std::{
    fs,
    num::ParseIntError,
    path::{Path, PathBuf},
};
use system_json::SystemJson;
use walkdir::WalkDir;

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];
const HAS_ENC_AUIDO_KEY: &str = "hasEncryptedAudio";
const HAS_ENC_IMG_KEY: &str = "hasEncryptedImages";
const ENCKEY_KEY: &str = "encryptionKey";

pub mod error;
pub mod prelude;
mod rpg_file;
mod system_json;
mod tests;

/// Represents an RpgMaker game.
#[derive(Debug)]
pub struct RpgGame {
    path: PathBuf,
    key: Vec<u8>,
    orig_key: String,
    system_json: SystemJson,
    verbose: bool,
    num_files: Option<usize>,
}

/// Configures how to process and store the decrypted files.
///
/// You can use this struct as a clap Subcommand by enabling
/// the `clap` feature.
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputSettings {
    /// Decrypts the game's files in next to the encrypted files (default)
    NextTo,

    /// Overwrites the games files with the decrypted ones.
    Replace,

    /// Leaves the game untouched, places files into given directory while maintining original dir structure.
    Output { dir: PathBuf },

    /// Same as output but flattens the dir structure
    Flatten { dir: PathBuf },
}

/// Represents the games encryption key as a raw string
/// (as stored in System.json) and as bytes that can
/// be used to decrypt a game.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgKey<'a> {
    pub string: &'a str,
    pub bytes: &'a [u8],
}

impl RpgGame {
    /// Attempt to create a new RpgGame from a given path.
    /// setting `verbose` to true will print decryption progress to stdout
    ///
    /// ## Example
    /// ```
    /// use librpgmaker::prelude::*;
    ///
    /// let game = RpgGame::new("path/to/game", false);
    /// ```
    pub fn new<P: AsRef<Path>>(path: P, verbose: bool) -> Result<Self, Error> {
        let system_json = Self::get_system_json(path.as_ref())?;
        let (key, orig_key) = Self::try_get_key(&system_json.data)?;

        Ok(Self {
            num_files: None,
            verbose,
            key,
            orig_key,
            system_json,
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Scans files in the game directory and returns a list of all files that can decrypted.
    ///
    /// This does not read the file contents, only filename.
    ///
    /// The result of this operation is cached and will be used to display the total amount
    /// of files left when decrypting (if verbose == true)
    pub fn scan_files(&mut self) -> Result<Vec<RpgFileType>, Error> {
        let files: Vec<_> = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|path| match path {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .filter_map(|entry| RpgFileType::scan(&entry.path()))
            .collect();

        self.num_files = Some(files.len());
        Ok(files)
    }

    /// Decrypt all files in the game directory.
    ///
    /// Returns the number of files decrypted or an error.
    ///
    /// When `verbose` is true, the decryption progress will be
    /// printed to stdout. The total number of files will only
    /// be displayed if `scan_files()` was run beforehand.
    pub fn decrypt_all(&mut self, output: &OutputSettings) -> Result<u64, Error> {
        let files = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|path| match path {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .filter_map(|entry| RpgFile::from_path(&entry.path()));

        let mut num_decrypted = 0;

        for file in files {
            num_decrypted += 1;

            match (self.num_files, self.verbose) {
                (Some(num_files), true) => {
                    println!(
                        "[{}/{}] {}",
                        num_decrypted,
                        num_files,
                        file.orig_path.display()
                    )
                }
                (None, true) => println!("[{}] {}", num_decrypted, file.orig_path.display()),
                _ => {}
            }

            let decrypted = file.decrypt(&self.key);

            let new_path = match output {
                OutputSettings::NextTo => file.new_path,

                OutputSettings::Replace => {
                    self.system_json.encrypted = false;
                    dbg!(&file.orig_path);
                    fs::remove_file(file.orig_path)?;
                    file.new_path
                }

                OutputSettings::Output { dir } => {
                    let new_path = dir.join(file.new_path.strip_prefix(&self.path)?);
                    fs::create_dir_all(&new_path.parent().expect("No parent"))?;
                    new_path
                }

                OutputSettings::Flatten { dir } => {
                    fs::create_dir_all(&dir)?;

                    let path_str = file
                        .new_path
                        .strip_prefix(&self.path)
                        .expect("no parent")
                        .to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "_");

                    let mut new_dir = dir.join(PathBuf::from(path_str));
                    new_dir.set_extension(file.file_type.to_extension());
                    new_dir
                }
            };

            fs::write(&new_path, decrypted)?;
        }

        self.system_json.write()?;

        Ok(num_decrypted)
    }

    /// Returns the game's decryption key
    pub fn get_key(&self) -> RpgKey {
        RpgKey {
            string: &self.orig_key,
            bytes: &self.key,
        }
    }

    /// Indicates if the game reports to be decrypted or not.
    pub fn is_encrypted(&self) -> bool {
        self.system_json.encrypted
    }

    fn try_get_key(system_json: &Value) -> Result<(Vec<u8>, String), Error> {
        fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
                .collect()
        }

        match system_json.get(ENCKEY_KEY) {
            Some(key) => match key.as_str() {
                Some(key) => Ok((decode_hex(key)?, key.to_owned())),
                None => Err(Error::SystemJsonInvalidKey {
                    key: key.to_string(),
                }),
            },
            None => Err(Error::SystemJsonKeyNotFound {
                key: ENCKEY_KEY.to_string(),
            }),
        }
    }

    fn get_system_json(path: &Path) -> Result<SystemJson, Error> {
        let system_paths: Vec<PathBuf> = SYS_JSON_PATHS
            .iter()
            .map(|x| path.join(PathBuf::from(x)))
            .filter(|path| path.exists())
            .collect();

        let system_path = match system_paths.is_empty() {
            true => return Err(Error::SystemJsonNotFound),
            false => system_paths
                .first()
                .expect("no first path even though checked"),
        };

        let system = fs::read_to_string(system_path)?;
        match serde_json::from_str::<Value>(&system) {
            Ok(v) => Ok(SystemJson {
                encrypted: check_encrypted(&v)?,
                data: v,
                path: system_path.to_owned(),
            }),
            Err(e) => Err(Error::SystemJsonInvalidJson(e)),
        }
    }
}

fn check_encrypted(value: &Value) -> Result<bool, Error> {
    let get_key = |key: &str| -> Result<bool, Error> {
        match value.get(key) {
            Some(val) => match val.as_bool() {
                Some(v) => Ok(v),
                None => {
                    return Err(Error::SystemJsonInvalidKey {
                        key: val.to_string(),
                    })
                }
            },
            None => {
                return Err(Error::SystemJsonKeyNotFound {
                    key: key.to_string(),
                })
            }
        }
    };

    let audio = get_key(HAS_ENC_AUIDO_KEY)?;
    let img = get_key(HAS_ENC_IMG_KEY)?;

    Ok(audio || img)
}
