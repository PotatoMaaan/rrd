use std::{fs, path::PathBuf};

use serde_json::Value;

use crate::{error::Error, HAS_ENC_AUIDO_KEY, HAS_ENC_IMG_KEY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemJson {
    pub data: Value,
    pub path: PathBuf,
    pub encrypted: bool,
}

impl SystemJson {
    pub fn set_decrypt(&mut self, encrypted: bool) -> Result<(), Error> {
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

    pub fn write(&mut self) -> Result<(), Error> {
        self.set_decrypt(self.encrypted)?;

        let data = self.data.to_string();
        Ok(fs::write(&self.path, data)?)
    }
}
