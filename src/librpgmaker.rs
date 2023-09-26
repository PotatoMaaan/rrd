use std::{
    fs,
    num::ParseIntError,
    path::{Path, PathBuf, StripPrefixError},
};

use serde_json::Value;
use walkdir::WalkDir;

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];

#[derive(Debug)]
pub struct RpgGame {
    path: PathBuf,
    key: Vec<u8>,
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
    fn set_decrypt(&mut self) -> Result<(), Error> {
        let mut set_key = |key: &str| -> Result<(), Error> {
            Ok(
                *self.data.get_mut(key).ok_or(Error::SystemJsonKeyNotFound)? =
                    Value::Bool(self.encrypted),
            )
        };

        set_key("hasEncryptedAudio")?;
        set_key("hasEncryptedImages")?;

        Ok(())
    }

    fn write(&mut self) -> Result<(), Error> {
        if self.encrypted {
            self.set_decrypt()?;
        }

        let data = self.data.to_string();
        Ok(fs::write(&self.path, data)?)
    }
}

#[derive(Debug)]
pub enum Error {
    SystemJsonNotFound,
    IoError(std::io::Error),
    SystemJsonInvalid(serde_json::Error),
    SystemJsonKeyNotFound,
    SystemJsonInvalidKey,
    StrixPrefixFailed(StripPrefixError),
    KeyParseError(ParseIntError),
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
    path: PathBuf,
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
        let mut path = path.to_path_buf();
        let _ = path.set_extension(ext);

        Some(Self {
            data,
            file_type,
            path,
        })
    }

    #[allow(unused)]
    pub fn from_parts(data: Vec<u8>, file_type: RpgFileType, path: PathBuf) -> Self {
        Self {
            data,
            file_type,
            path,
        }
    }

    pub fn decrypt(&self, key: &[u8]) -> Vec<u8> {
        fn xor(data: &[u8], key: &[u8]) -> Vec<u8> {
            let mut result = vec![];

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputSettings {
    InPlace,
    Specific { dir: PathBuf },
    Flatten { dir: PathBuf },
}

impl RpgGame {
    /// Creates a new RpgGame instance from a given path
    pub fn new<P: AsRef<Path>>(path: P, verbose: bool) -> Result<Self, Error> {
        let system_json = Self::get_system_json(path.as_ref())?;
        let key = Self::get_key(&system_json.data)?;

        Ok(Self {
            num_files: None,
            verbose,
            key,
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
    pub fn decrypt_all(&mut self, output: &OutputSettings) -> Result<u32, Error> {
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
                    println!("[{}/{}] {}", num_decrypted, num_files, file.path.display())
                }
                (None, true) => println!("[{}] {}", num_decrypted, file.path.display()),
                _ => {}
            }

            let decrypted = file.decrypt(&self.key);

            let new_path = match output {
                OutputSettings::InPlace => file.path,
                OutputSettings::Specific { dir } => {
                    let new_path = dir.join(file.path.strip_prefix(&self.path)?);
                    fs::create_dir_all(&new_path.parent().expect("No parent"))?;
                    new_path
                }
                OutputSettings::Flatten { dir } => {
                    fs::create_dir_all(&dir)?;

                    let path_str = file
                        .path
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
        self.system_json.encrypted = false;
        self.system_json.write()?;

        Ok(num_decrypted)
    }

    fn get_key(system_json: &Value) -> Result<Vec<u8>, Error> {
        fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
                .collect()
        }

        match system_json.get("encryptionKey") {
            Some(key) => match key.as_str() {
                Some(key) => Ok(decode_hex(key)?),
                None => Err(Error::SystemJsonInvalidKey),
            },
            None => Err(Error::SystemJsonKeyNotFound),
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
        match serde_json::from_str(&system) {
            Ok(v) => Ok(SystemJson {
                encrypted: false,
                data: v,
                path: system_path.to_owned(),
            }),
            Err(e) => Err(Error::SystemJsonInvalid(e)),
        }
    }
}
