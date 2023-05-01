use clap::Parser;
use futures::future::join_all;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use std::{
    fs::{self, read, File},
    io::Write,
    num::ParseIntError,
    path::PathBuf,
};
use tokio::task::JoinHandle;
use walkdir::WalkDir;

/// Decrypt files encryped by RPMVs default encryprion
#[derive(Parser)]
struct Cli {
    /// The game directory containing the main executable file
    directory: std::path::PathBuf,
    /// Keep the original (encrypted) file next to the decrypted files
    #[arg(short, long)]
    keep_original: bool,
    /// The directory where decrypted files are output to relative to the current directory. This automatically keeps the encrypted files in place. If not specified, the files will be alongside the encrypted ones
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let base_path = args.directory;

    let _base_dir = match base_path.read_dir() {
        Ok(dir) => dir,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                println!(
                    "The provided directory \"{}\" does not exist",
                    base_path.into_os_string().into_string().unwrap()
                );
                std::process::exit(1);
            }
            std::io::ErrorKind::PermissionDenied => {
                println!("No permissionn to read the directory");
                std::process::exit(1);
            }
            _ => {
                println!("Error while reading the directory: {:?}", err);
                std::process::exit(1);
            }
        },
    };

    println!("Searching for decryptable files...");
    let mut total_file_amnount = 0;
    for path in WalkDir::new(base_path.clone()) {
        let path = path.expect("Failed to get dir");
        let path = restore_filename(path.path().into());
        if let Some(_path) = path {
            total_file_amnount += 1;
        }
    }
    if total_file_amnount > 0 {
        println!("Found {} decryptable files!", total_file_amnount);
    } else {
        println!("Did not find any decryptable files, exiting...");
        return Ok(());
    }

    let system_json_path = base_path.join("www/data/System.json");
    let mut system_json = get_system_json(system_json_path.clone())
        .map_err(|_e| {
            println!("System.json not found or it was invalid!");
            exit(1);
        })
        .unwrap();

    if system_json["encryptionKey"].is_null() {
        panic!("Key in System.json is invalid")
    }

    let encryption_key = system_json["encryptionKey"].as_str().unwrap();
    println!("Found encryption key: {}", encryption_key);
    let encryption_key = decode_hex(&encryption_key).expect("Invalid key in System.json");

    let num_dec_files = Arc::new(AtomicUsize::new(0));
    let mut handles: Vec<JoinHandle<()>> = vec![];

    println!("Starting decryption...");
    let start_time = Instant::now();
    for entry in WalkDir::new(base_path.clone()) {
        let entry = entry.expect("Failed to get dir entry");
        let entry = entry.path().to_path_buf();
        let restored_path = restore_filename(entry.clone());
        let new_path = match restored_path {
            Some(path) => match args.output.clone() {
                Some(ref dir) => {
                    let out_path: PathBuf = dir.join(path.strip_prefix(&base_path).unwrap());
                    let _ = fs::create_dir_all(out_path.parent().expect("Has no parent"))
                        .expect("Failed to mkdir");
                    out_path
                }
                None => path,
            },
            None => {
                continue;
            }
        };

        let output_clone = args.output.clone();
        let encryption_key_clone = encryption_key.clone();
        let num_dec_clone = Arc::clone(&num_dec_files);
        let handle = tokio::spawn(async move {
            match decrypt_file(entry.clone(), &encryption_key_clone, &new_path) {
                Ok(_) => {
                    num_dec_clone.fetch_add(1, Ordering::SeqCst);
                    println!(
                        "[{}/{}] Decrypting: {}\n\t-> {}",
                        num_dec_clone.load(Ordering::SeqCst),
                        total_file_amnount,
                        &entry.display(),
                        new_path.display()
                    );

                    if !args.keep_original {
                        match output_clone {
                            None => std::fs::remove_file(&entry).expect(
                                format!("Failed to remove file: {}", &entry.display()).as_str(),
                            ),
                            _ => {}
                        }
                    }
                }
                Err(err) => {
                    println!(
                        "WARNING: Failed to decrypt: {} :{:#?}",
                        &entry.display(),
                        err
                    );
                }
            }
        });
        handles.push(handle);
    }
    join_all(handles).await;

    println!(
        "\n\nDecrypted {} files in {:.2?}.",
        num_dec_files.load(Ordering::SeqCst),
        start_time.elapsed()
    );
    // Only write to System.json if the files were actually decrypted
    if !args.keep_original && args.output == None {
        println!("Updating System.json");
        system_json["hasEncryptedAudio"] = serde_json::Value::Bool(false);
        system_json["hasEncryptedImages"] = serde_json::Value::Bool(false);
        write_json(system_json, system_json_path).expect("Failed to write to System.json");
    }

    println!("Game decrypted!");

    Ok(())
}

fn rpgmv_xor_decrypt(data: Vec<u8>, key: &Vec<u8>) -> Result<Vec<u8>, String> {
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

fn decrypt_file(
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

fn restore_filename(mut path: PathBuf) -> Option<PathBuf> {
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

fn get_system_json(path: PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(path)?;

    // Remove BOM as serde_json cannot handle it
    let system_json: serde_json::Value =
        serde_json::from_str(&file_content.trim_start_matches("\u{feff}"))?;

    Ok(system_json)
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn write_json(json: serde_json::Value, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new().write(true).truncate(true).open(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &json)?;
    Ok(())
}
