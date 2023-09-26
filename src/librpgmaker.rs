use std::{
    fs,
    num::ParseIntError,
    path::{Path, PathBuf, StripPrefixError},
};

use clap::Subcommand;
use serde_json::Value;
use walkdir::WalkDir;

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];
const HAS_ENC_AUIDO_KEY: &str = "hasEncryptedAudio";
const HAS_ENC_IMG_KEY: &str = "hasEncryptedImages";
const KEY_KEY: &str = "encryptionKey";

#[derive(Debug)]
pub struct RpgGame {
    path: PathBuf,
    key: Vec<u8>,
    orig_key: String,
    system_json: SystemJson,
    verbose: bool,
    num_files: Option<usize>,
}

#[derive(Debug)]
struct SystemJson {
    data: Value,
    path: PathBuf,
    encrypted: bool,
}

impl SystemJson {
    fn set_decrypt(&mut self, encrypted: bool) -> Result<(), Error> {
        let mut set_key = |key: &str| -> Result<(), Error> {
            let json_key = self.data.get_mut(key).ok_or(Error::SystemJsonKeyNotFound {
                key: key.to_string(),
            })?;

            Ok(*json_key = Value::Bool(encrypted))
        };

        set_key(HAS_ENC_AUIDO_KEY)?;
        set_key(HAS_ENC_IMG_KEY)?;
        self.encrypted = encrypted;

        Ok(())
    }

    fn write(&mut self) -> Result<(), Error> {
        self.set_decrypt(self.encrypted)?;

        let data = self.data.to_string();
        Ok(fs::write(&self.path, data)?)
    }
}

#[derive(Debug)]
pub enum Error {
    SystemJsonNotFound,
    IoError(std::io::Error),
    SystemJsonInvalid(serde_json::Error),
    SystemJsonKeyNotFound { key: String },
    SystemJsonInvalidKey { key: String },
    StrixPrefixFailed(StripPrefixError),
    KeyParseError(ParseIntError),
    OutputDirExists,
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::KeyParseError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<StripPrefixError> for Error {
    fn from(value: StripPrefixError) -> Self {
        Self::StrixPrefixFailed(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RpgFileType {
    RpgAudio,
    RpgVideo,
    RpgImage,
}

impl RpgFileType {
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

    pub fn to_extension(&self) -> String {
        match self {
            RpgFileType::RpgAudio => "ogg",
            RpgFileType::RpgVideo => "m4a",
            RpgFileType::RpgImage => "png",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgFile {
    data: Vec<u8>,
    file_type: RpgFileType,
    new_path: PathBuf,
    orig_path: PathBuf,
}

impl RpgFile {
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_type = RpgFileType::scan(&path)?;

        let data = match fs::read(&path) {
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
    pub fn from_parts(data: Vec<u8>, file_type: RpgFileType, orig_path: PathBuf) -> Self {
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
        return plaintext;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Subcommand)]
pub enum OutputSettings {
    /// Decrypts the game's files in place (default)
    InPlace,

    /// Overwrites the games files with the decrypted ones.
    Overwrite,

    /// Leaves the game untouched, places files into given directory while maintining original dir structure.
    Specific { dir: PathBuf },

    /// Same as specific but flattens the dir structure
    Flatten { dir: PathBuf },
}

pub struct RpgKey<'a> {
    pub string: &'a str,
    pub bytes: &'a [u8],
}

impl RpgGame {
    /// Creates a new RpgGame instance from a given path
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

    /// Scans files in the game directory and returns a list of all files that can be decrypted
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
    /// Returns the number of files decrypted or an error
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
                OutputSettings::InPlace => file.new_path,

                OutputSettings::Overwrite => {
                    self.system_json.encrypted = false;
                    dbg!(&file.orig_path);
                    fs::remove_file(file.orig_path)?;
                    file.new_path
                }

                OutputSettings::Specific { dir } => {
                    if dir.exists() {
                        return Err(Error::OutputDirExists);
                    }

                    let new_path = dir.join(file.new_path.strip_prefix(&self.path)?);
                    fs::create_dir_all(&new_path.parent().expect("No parent"))?;
                    new_path
                }

                OutputSettings::Flatten { dir } => {
                    if dir.exists() {
                        return Err(Error::OutputDirExists);
                    }

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

    pub fn get_key(&self) -> RpgKey {
        RpgKey {
            string: &self.orig_key,
            bytes: &self.key,
        }
    }

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

        match system_json.get(KEY_KEY) {
            Some(key) => match key.as_str() {
                Some(key) => Ok((decode_hex(key)?, key.to_owned())),
                None => Err(Error::SystemJsonInvalidKey {
                    key: key.to_string(),
                }),
            },
            None => Err(Error::SystemJsonKeyNotFound {
                key: KEY_KEY.to_string(),
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
            Err(e) => Err(Error::SystemJsonInvalid(e)),
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

    if audio != img {
        panic!("System.json indicates that audio and img encryption is not the same, this is currenty unsupported.")
    }

    Ok(audio)
}
