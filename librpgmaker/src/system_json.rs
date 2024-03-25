use std::{
    fs,
    num::ParseIntError,
    path::{Path, PathBuf},
};

use crate::{Encryption, EncryptionState};

const HAS_ENC_AUIDO_KEY: &str = "hasEncryptedAudio";
const HAS_ENC_IMG_KEY: &str = "hasEncryptedImages";
const ENCKEY_KEY: &str = "encryptionKey";
const GAME_TITLE_KEY: &str = "gameTitle";

const SYS_JSON_PATHS: &[&str] = &["www/data/System.json", "data/System.json"];

#[derive(Debug)]
pub struct SystemJson {
    path: PathBuf,

    // This takes some memory, but I'd argue it's better than parsing
    // the file every time we need to work with it.
    data: serde_json::Value,
}

impl SystemJson {
    pub fn find_system_json(dir: &Path) -> crate::error::Result<Self> {
        let sys_json = SYS_JSON_PATHS
            .iter()
            .map(|x| dir.join(x))
            .find_map(|path| fs::File::open(&path).ok().map(|f| (path, f)));

        if let Some((path, sys_json_file)) = sys_json {
            let data = serde_json::from_reader::<_, serde_json::Value>(sys_json_file)
                .map_err(|e| crate::Error::SystemJsonInvalidJson(e))?;

            Ok(Self { path, data })
        } else {
            Err(crate::Error::SystemJsonNotFound)
        }
    }

    pub fn set_encryption_state(&mut self, state: Encryption) -> crate::error::Result<()> {
        let state = match state {
            Encryption::Encrypted(_) => true,
            Encryption::Decrypted(_) => false,
        };

        let audio = self.data.get_mut(HAS_ENC_AUIDO_KEY).ok_or_else(|| {
            crate::Error::SystemJsonKeyNotFound {
                key: HAS_ENC_AUIDO_KEY.to_owned(),
            }
        })?;
        *audio = serde_json::Value::Bool(state);

        let imgs = self.data.get_mut(HAS_ENC_IMG_KEY).ok_or_else(|| {
            crate::Error::SystemJsonKeyNotFound {
                key: HAS_ENC_IMG_KEY.to_owned(),
            }
        })?;
        *imgs = serde_json::Value::Bool(state);

        fs::write(
            &self.path,
            serde_json::to_string(&self.data)
                .map_err(|e| crate::Error::SystemJsonInvalidJson(e))?,
        )
        .map_err(|e| crate::Error::IoError {
            err: e,
            file: self.path.to_path_buf(),
        })?;

        Ok(())
    }

    pub fn game_title(&self) -> Option<&str> {
        self.data.get(GAME_TITLE_KEY)?.as_str()
    }

    pub fn key(&self) -> crate::error::Result<Vec<u8>> {
        fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
                .collect()
        }

        let key_txt = self
            .data
            .get(ENCKEY_KEY)
            .ok_or_else(|| crate::Error::SystemJsonKeyNotFound {
                key: ENCKEY_KEY.to_owned(),
            })?
            .as_str()
            .ok_or_else(|| crate::Error::SystemJsonInvalidKey {
                key: ENCKEY_KEY.to_owned(),
            })?;

        let key_bytes = decode_hex(key_txt)?;

        Ok(key_bytes)
    }

    pub fn is_encrypted(&self) -> bool {
        let has_enc_audio = self
            .data
            .get(HAS_ENC_AUIDO_KEY)
            .unwrap_or_else(|| &serde_json::Value::Bool(false))
            .as_bool()
            .unwrap_or(false);

        let has_enc_imgs = self
            .data
            .get(HAS_ENC_IMG_KEY)
            .unwrap_or_else(|| &serde_json::Value::Bool(false))
            .as_bool()
            .unwrap_or(false);

        has_enc_audio || has_enc_imgs
    }
}
