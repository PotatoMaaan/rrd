use clap::Parser;
use cli::*;
use std::{
    fs::{create_dir, create_dir_all},
    path::PathBuf,
    process::exit,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::task::JoinHandle;
use util::*;
use uuid::Uuid;
use walkdir::WalkDir;

mod cli;
mod util;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let base_path = args.directory;
    let flatten_paths = match args.output {
        Some(_) => args.flatten_paths,
        None => false,
    };

    if let Some(path) = &args.output {
        if path.exists() {
            eprintln!("ERROR: Output path {} already exists!", path.display());
            exit(0);
        }
    }

    println!("Searching for decryptable files...");
    let mut total_file_amnount = 0;
    for path in WalkDir::new(base_path.clone()) {
        let path = path.expect("Failed to get dir");
        let path = restore_filename(path.path().into());
        if path.is_some() {
            total_file_amnount += 1;
        }
    }

    if total_file_amnount > 0 {
        println!("Found {} decryptable files", total_file_amnount);

        // Exit if just scanning for files
        if args.scan {
            exit(0);
        }
    } else {
        eprintln!("ERROR: Did not find any decryptable files");
        exit(1);
    }

    let path1 = base_path.join("www/data/System.json");
    let path2 = base_path.join("data/System.json");
    let system_json_path;

    if path1.exists() {
        system_json_path = path1
    } else if path2.exists() {
        system_json_path = path2
    } else {
        eprintln!("ERROR: System.json not found");
        exit(1);
    }

    let mut system_json = get_system_json(system_json_path.clone()).expect("Invalid System.json");
    let encryption_key = system_json["encryptionKey"]
        .as_str()
        .expect("No encryption key in System.json!");
    println!("Found encryption key: {}", encryption_key);

    // Exit if the key option was given
    if args.key {
        exit(0);
    }

    let encryption_key = decode_hex(encryption_key).expect("Invalid key in System.json");
    let num_dec_files = Arc::new(AtomicUsize::new(0));
    let mut handles: Vec<JoinHandle<()>> = vec![];

    println!("Starting decryption...");
    let start_time = Instant::now();

    for entry in WalkDir::new(base_path.clone()) {
        let entry = entry.expect("Failed to get dir entry");
        let entry = entry.path().to_path_buf();
        let restored_path = restore_filename(entry.clone());

        let new_path = match (restored_path, args.output.clone(), flatten_paths) {
            // An output path is specified, construct an output path relative to the specified one
            (Some(path), Some(ref dir), true) => {
                let mut out_path = dir.join(Uuid::new_v4().to_string());
                if let Some(p) = path.extension() {
                    out_path.set_extension(p);
                };
                if !dir.exists() {
                    create_dir(dir).expect("Failed to mkdir");
                }
                out_path
            }
            (Some(path), Some(ref dir), false) => {
                let out_path: PathBuf = dir.join(path.strip_prefix(&base_path).unwrap());
                create_dir_all(out_path.parent().expect("Has no parent")).expect("Failed to mkdir");
                out_path
            }
            (Some(path), None, _) => path,
            _ => {
                continue;
            }
        };

        // Variables need to be cloned to move them into the task
        let output_clone = args.output.clone();
        let encryption_key_clone = encryption_key.clone();
        let num_dec_clone = Arc::clone(&num_dec_files);

        // Create new tokio task for each file, this is much faster than decrypting the files in order
        let handle = tokio::spawn(async move {
            if !args.quiet {
                println!(
                    "[{}/{}] Decrypting: {}\n\t-> {}",
                    num_dec_clone.load(Ordering::SeqCst),
                    total_file_amnount,
                    &entry.display(),
                    new_path.display()
                );
            }

            if let Err(e) = decrypt_file(entry.clone(), &encryption_key_clone, &new_path) {
                println!("WARNING: Failed to decrypt: {} :{:?}", &entry.display(), e);
                return;
            }

            num_dec_clone.fetch_add(1, Ordering::SeqCst);

            if !args.keep_original && output_clone.is_none() {
                std::fs::remove_file(&entry)
                    .unwrap_or_else(|_| panic!("Failed to remove file: {}", &entry.display()));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Failed to await handle");
    }

    println!(
        "\n\nDecrypted {} files in {:.2?}.",
        num_dec_files.load(Ordering::SeqCst),
        start_time.elapsed()
    );

    // Only write to System.json if the files were actually decrypted in place
    if !args.keep_original && args.output.is_none() {
        println!("Updating System.json");
        system_json["hasEncryptedAudio"] = serde_json::Value::Bool(false);
        system_json["hasEncryptedImages"] = serde_json::Value::Bool(false);
        write_json(system_json, system_json_path).expect("Failed to write to System.json");
    }

    println!("Game decrypted!");
}
