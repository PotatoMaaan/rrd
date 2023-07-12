use std::{
    fs::{read, write, File},
    io::Write,
    num::ParseIntError,
    path::PathBuf,
};

pub fn rpgmv_xor_decrypt(data: Vec<u8>, key: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let key_len = key.len();
    let data_len = data.len();
    for i in 0..data_len {
        result.push(
            data.get(i).ok_or("Invalid Index")? ^ key.get(i % key_len).ok_or("Invalid index")?,
        );
    }
    Ok(result)
}

pub fn decrypt_file(
    file_path: PathBuf,
    key: &Vec<u8>,
    new_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = read(&file_path)?;
    let file = file[16..].to_vec();
    let cyphertext = file[..16].to_vec();
    let mut plaintext = rpgmv_xor_decrypt(cyphertext, key)?;
    let mut file = file[16..].to_vec();
    //println!("{:?}", plaintext);
    plaintext.append(&mut file);
    let mut new_file = File::create(&new_path)?;
    new_file.write_all(&plaintext)?;
    Ok(())
}

pub fn restore_filename(mut path: PathBuf) -> Option<PathBuf> {
    let extension = path.extension()?.to_owned();
    match extension.to_str()? {
        "rpgmvo" => {
            path.set_extension("ogg");
            Some(path)
        }
        "ogg_" => {
            path.set_extension("ogg");
            Some(path)
        }
        "rpgmvm" => {
            path.set_extension("m4a");
            Some(path)
        }
        "m4a_" => {
            path.set_extension("m4a");
            Some(path)
        }
        "rpgmvp" => {
            path.set_extension("png");
            return Some(path);
        }
        "png_" => {
            path.set_extension("png");
            return Some(path);
        }
        _ => None,
    }
}

pub fn get_system_json(path: PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(path)?;

    // Remove BOM if present as serde_json cannot handle it
    let system_json: serde_json::Value =
        serde_json::from_str(&file_content.trim_start_matches("\u{feff}"))?;

    Ok(system_json)
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn write_json(
    json: serde_json::Value,
    path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let jstr = serde_json::to_string(&json)?;
    write(path, jstr)?;
    Ok(())
}
