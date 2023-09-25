use std::{
    fs,
    path::{Path, PathBuf, StripPrefixError},
};

use serde_json::Value;
use uuid::Uuid;
use walkdir::WalkDir;

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];

#[derive(Debug)]
pub struct RpgGame {
    path: PathBuf,
    key: Vec<u8>,
    system_json: Value,
}

#[derive(Debug)]
pub enum Error {
    SystemJsonNotFound,
    IoError(std::io::Error),
    SystemJsonInvalid(serde_json::Error),
    SystemJsonNoKey,
    SystemJsonInvalidKey,
    StrixPrefixFailed(StripPrefixError),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn decrypt(&self, key: &[u8]) -> Vec<u8> {
        fn xor(data: &[u8], key: &[u8]) -> Vec<u8> {
            let mut result = vec![];

            for i in 0..data.len() {
                result.push(data[i] ^ key[i % key.len()]);
            }

            result
        }

        let file = &self.data;
        let file = &file[16..];
        let cyphertext = &file[..16];
        let mut plaintext = xor(cyphertext, key);
        let mut file = file[16..].to_vec();
        plaintext.append(&mut file);
        plaintext
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputSettings {
    InPlace,
    Specific { dir: PathBuf },
    Flatten { dir: PathBuf },
}

impl RpgGame {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let system_json = Self::get_system_json(path.as_ref())?;
        let key = Self::get_key(&system_json)?;

        Ok(Self {
            key,
            system_json,
            path: path.as_ref().to_path_buf(),
        })
    }

    pub fn scan_files(&self) -> Result<Vec<RpgFileType>, Error> {
        Ok(WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|path| match path {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .filter_map(|entry| RpgFileType::scan(&entry.path()))
            .collect())
    }

    pub fn decrypt_all(&self, output: &OutputSettings) -> Result<(), Error> {
        let files = WalkDir::new(&self.path)
            .into_iter()
            .filter_map(|path| match path {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .filter_map(|entry| RpgFile::from_path(&entry.path()));

        for file in files {
            let new_dat = file.decrypt(&self.key);

            let new_path = match output {
                OutputSettings::InPlace => file.path,
                OutputSettings::Specific { dir } => dir.join(file.path.strip_prefix(&self.path)?),
                OutputSettings::Flatten { dir } => {
                    let path_str = dir
                        .to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "_");
                    let uuid: String = Uuid::new_v4().to_string().chars().take(10).collect();
                    let path_str = PathBuf::from(format!("{}_{}", path_str, uuid));
                    dir.join(path_str)
                }
            };

            fs::write(&new_path, new_dat)?;
        }

        Ok(())
    }

    fn get_key(system_json: &Value) -> Result<Vec<u8>, Error> {
        match system_json.get("encryptionKey") {
            Some(key) => match key.as_str() {
                Some(key) => Ok(key.bytes().collect()),
                None => Err(Error::SystemJsonInvalidKey),
            },
            None => Err(Error::SystemJsonNoKey),
        }
    }

    fn get_system_json(path: &Path) -> Result<Value, Error> {
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
            Ok(v) => Ok(v),
            Err(e) => Err(Error::SystemJsonInvalid(e)),
        }
    }
}
