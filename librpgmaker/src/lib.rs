//! A Library to interact with and decrypt RpgMaker games.
//! To get started, see the `RpgGame` struct.

use error::Error;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use rpg_file::{Decrypted, RpgFile, RpgFileType};
use serde_json::Value;
use std::{
    cell::Cell,
    fs,
    num::ParseIntError,
    path::{Path, PathBuf},
    sync::atomic::AtomicI64,
    time::{Duration, Instant},
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
    game_path: PathBuf,
    key_bytes: Vec<u8>,
    key_str: String,
    system_json: SystemJson,
    verbose: bool,
    num_files: Cell<Option<usize>>,
}

/// Configures how to process and store the decrypted files.
///
/// You can use this struct as a clap Subcommand by enabling
/// the `clap` feature.
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputSettings {
    /// Decrypts the game's files next to the encrypted files
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
    /// Attempt to create a new `RpgGame` from a given path.
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
            verbose,
            key_bytes: key,
            key_str: orig_key,
            system_json,
            game_path: path.as_ref().to_path_buf(),
            num_files: Cell::new(None),
        })
    }

    /// Scans files in the game directory and returns a list of all files that can decrypted.
    ///
    /// This does not read the file contents, only filename.
    ///
    /// The result of this operation is cached and will be used to display the total amount
    /// of files left when decrypting (if verbose == true)
    pub fn scan_files(&self) -> Result<Vec<RpgFileType>, Error> {
        let files: Vec<_> = WalkDir::new(&self.game_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|entry| RpgFileType::from_encrypted_path(entry.path()))
            .collect();

        self.num_files.set(Some(files.len()));
        Ok(files)
    }

    /// Decrypt all files in the game directory.
    ///
    /// Returns a Vec of results that contain eiter the decryption `Duration`
    /// and the filename, or an Error.
    ///
    /// When `verbose` is true, the decryption progress will be
    /// printed to stdout. The total number of files will only
    /// be displayed if `scan_files()` was run beforehand.
    pub fn decrypt_all(
        &mut self,
        output: &OutputSettings,
    ) -> Result<Vec<Result<(Duration, PathBuf), Error>>, Error> {
        use std::sync::atomic::Ordering as Ord;

        // Only scan files if not previously already done
        let num_files = self.num_files.get().unwrap_or(self.scan_files()?.len());
        let num_decrypted = AtomicI64::new(0);

        let results = WalkDir::new(&self.game_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| RpgFile::from_path_encrypted(entry.path()))
            .map(|file| -> Result<(Duration, PathBuf), Error> {
                let start_time = Instant::now();

                let file = file.decrypt(&self.key_bytes)?;
                let new_path = create_path_from_output(output, &file, &self.game_path)?;

                num_decrypted.fetch_add(1, Ord::SeqCst);
                if self.verbose {
                    print_progress(
                        num_files,
                        num_decrypted.load(Ord::SeqCst) as u64,
                        &file,
                        &new_path,
                    );
                }

                fs::write(&new_path, file.data())?;

                Ok((start_time.elapsed(), file.encrypted_path().to_path_buf()))
            })
            .collect::<Vec<_>>();

        // in case the files were decrypted in place, we need to update system.json
        if output == &OutputSettings::Replace {
            self.system_json.encrypted = false;
            self.system_json.write()?;
        }

        Ok(results)
    }

    pub fn encrypt_all(&mut self) -> Result<Vec<Result<(), Error>>, Error> {
        let num_files = self.num_files.get().unwrap_or(self.scan_files()?.len());

        let results = WalkDir::new(&self.game_path)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| RpgFile::from_path_decrypted(entry.path()))
            .map(|file| -> Result<(), Error> {
                let file = file.encrypt(&self.key_bytes)?;

                fs::write(&file.encrypted_path(), file.data())?;

                Ok(())
            })
            .collect::<Vec<_>>();

        self.system_json.encrypted = true;
        self.system_json.write()?;

        Ok(results)
    }

    /// Returns the game's decryption key
    #[must_use]
    pub fn get_key(&self) -> RpgKey {
        RpgKey {
            string: &self.key_str,
            bytes: &self.key_bytes,
        }
    }

    /// Indicates if the game reports to be decrypted or not.
    #[inline]
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        self.system_json.encrypted
    }

    fn try_get_key(system_json: &Value) -> Result<(Vec<u8>, String), Error> {
        match system_json.get(ENCKEY_KEY) {
            Some(key) => match key.as_str() {
                Some(key) => Ok((decode_hex(key)?, key.to_owned())),
                None => Err(Error::SystemJsonInvalidKey {
                    key: key.to_string(),
                }),
            },
            None => Err(Error::NotEncrypted),
        }
    }

    fn get_system_json(path: &Path) -> Result<SystemJson, Error> {
        let system_paths: Vec<PathBuf> = SYS_JSON_PATHS
            .iter()
            .map(|x| path.join(PathBuf::from(x)))
            .filter(|path| path.exists())
            .collect();

        let Some(system_path) = system_paths.get(0) else {
            return Err(Error::SystemJsonNotFound);
        };

        let system = fs::read_to_string(system_path)?;
        match serde_json::from_str::<Value>(&system) {
            Ok(v) => Ok(SystemJson {
                encrypted: check_encrypted(&v)?,
                data: v,
                path: system_path.clone(),
            }),
            Err(e) => Err(Error::SystemJsonInvalidJson(e)),
        }
    }
}

fn check_encrypted(value: &Value) -> Result<bool, Error> {
    let get_key = |key: &str| -> Result<bool, Error> {
        match value.get(key).unwrap_or(&Value::Bool(false)).as_bool() {
            Some(v) => Ok(v),
            None => Err(Error::SystemJsonInvalidKey {
                key: key.to_string(),
            }),
        }
    };

    let audio = get_key(HAS_ENC_AUIDO_KEY)?;
    let img = get_key(HAS_ENC_IMG_KEY)?;

    Ok(audio || img)
}

fn create_path_from_output<T>(
    output: &OutputSettings,
    file: &RpgFile<T>,
    game_path: &Path,
) -> Result<PathBuf, Error> {
    let new_path = match output {
        OutputSettings::NextTo => file.decrypted_path().to_path_buf(),

        OutputSettings::Replace => {
            fs::remove_file(file.encrypted_path())?;
            file.decrypted_path().to_path_buf()
        }

        OutputSettings::Output { dir } => {
            let new_path = dir.join(file.decrypted_path().strip_prefix(game_path)?);
            fs::create_dir_all(new_path.parent().expect("No parent"))?;
            new_path.to_path_buf()
        }

        OutputSettings::Flatten { dir } => {
            fs::create_dir_all(dir)?;

            // FIXME: if there are 2 files with a name that is only different due to non urf-8
            // characters, this will overwrite the file that came first with later ones
            // because to_string_lossy() discards any non utf-8 chars.
            //
            // Neither OsStr or OsString have a replace() method. the bstr crate would help here,
            // but adding a whole new crate just for this does not seem worth it.
            let path_str = file
                .decrypted_path() // test_files/game/www/img/test.png
                .strip_prefix(game_path) // www/img/test.png
                .expect("no parent")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "_"); // www_img_test.png

            dir.join(PathBuf::from(path_str)) // output_dir/www_img_test.png
        }
    };

    Ok(new_path.clone())
}

fn print_progress<T>(num_files: usize, num_decrypted: u64, file: &RpgFile<T>, new_path: &Path) {
    println!(
        "[{}/{}] {}\n  -> {}",
        num_decrypted,
        num_files,
        file.encrypted_path().display(),
        new_path.display()
    );
}

/// A utility function to turn a string of "hex bytes" into actual bytes.
///
/// # Examples
/// ```
/// use librpgmaker::decode_hex;
///
/// let string = "0f1a2b"; // valid
/// let hex = decode_hex(string).expect("invalid string");
///
/// assert_eq!(hex, vec![0x0f, 0x1a, 0x2b]);
///
/// let string = "0x0f3b"; // invalid
/// let result = decode_hex(string);
///
/// assert!(result.is_err());
/// ```
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// The XOR function used by RpgMaker
#[inline]
pub fn rpg_xor(data: &mut [u8], key: &[u8]) {
    data.iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}
