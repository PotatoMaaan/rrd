#![cfg(test)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tempdir::TempDir;

use crate::{
    create_path_from_output,
    rpg_file::{RpgFile, RpgFileType},
    OutputSettings,
};

const IMG_ENC: &[u8] = &[
    82, 80, 71, 77, 86, 0, 0, 0, 0, 3, 1, 0, 0, 0, 0, 0, 134, 95, 65, 72, 2, 5, 21, 5, 15, 15, 15,
    2, 70, 71, 75, 93, 0, 0, 0, 12, 0, 0, 0, 12, 8, 2, 0, 0, 0, 217, 23, 203, 176, 0, 0, 0, 9, 112,
    72, 89, 115, 0, 0, 14, 196, 0, 0, 14, 196, 1, 149, 43, 14, 27, 0, 0, 1, 146, 73, 68, 65, 84,
    24, 149, 5, 193, 201, 110, 211, 64, 24, 0, 224, 127, 22, 123, 28, 71, 73, 28, 167, 16, 40, 13,
    106, 47, 4, 164, 80, 129, 132, 56, 34, 122, 232, 145, 7, 224, 57, 81, 1, 169, 66, 8, 113, 233,
    137, 27, 82, 131, 72, 211, 98, 199, 137, 227, 165, 139, 221, 120, 38, 179, 241, 125, 232, 231,
    143, 239, 24, 97, 226, 16, 2, 6, 64, 223, 215, 183, 139, 127, 191, 171, 122, 97, 129, 3, 24,
    161, 164, 176, 132, 230, 25, 143, 163, 56, 8, 58, 251, 251, 123, 214, 218, 166, 113, 214, 5,
    40, 195, 44, 72, 64, 68, 106, 162, 177, 161, 139, 56, 154, 78, 207, 159, 142, 70, 97, 56, 232,
    7, 65, 183, 231, 7, 225, 104, 118, 153, 103, 101, 102, 141, 194, 132, 32, 98, 41, 70, 246, 197,
    120, 220, 233, 6, 85, 181, 137, 162, 228, 219, 233, 169, 5, 253, 104, 175, 13, 32, 168, 139,
    132, 104, 36, 215, 244, 98, 254, 167, 23, 132, 26, 144, 20, 69, 158, 165, 87, 151, 179, 188,
    72, 63, 62, 255, 144, 230, 220, 72, 173, 184, 228, 114, 75, 215, 73, 124, 93, 172, 122, 221,
    129, 146, 58, 75, 150, 204, 214, 207, 158, 116, 117, 30, 121, 247, 27, 131, 192, 104, 137, 148,
    164, 102, 187, 109, 154, 202, 52, 155, 221, 199, 7, 7, 111, 222, 90, 177, 106, 217, 77, 149,
    252, 5, 164, 1, 99, 176, 138, 130, 166, 174, 227, 0, 35, 204, 109, 107, 69, 227, 69, 34, 164,
    190, 173, 133, 207, 156, 201, 225, 107, 207, 119, 163, 249, 20, 137, 27, 178, 251, 96, 71, 52,
    188, 170, 107, 175, 229, 150, 215, 43, 5, 128, 153, 63, 62, 124, 21, 167, 57, 215, 246, 221,
    241, 241, 175, 179, 51, 50, 28, 236, 168, 173, 124, 56, 28, 50, 70, 215, 105, 214, 112, 193,
    27, 158, 44, 23, 8, 163, 247, 71, 71, 39, 95, 191, 52, 213, 29, 85, 198, 106, 107, 148, 133,
    201, 228, 101, 89, 150, 119, 203, 10, 144, 161, 24, 194, 126, 255, 211, 201, 231, 249, 213,
    133, 139, 129, 74, 164, 141, 177, 231, 179, 89, 113, 83, 50, 143, 73, 163, 49, 64, 203, 111, 3,
    118, 60, 230, 183, 253, 142, 50, 242, 63, 57, 78, 240, 199, 38, 234, 88, 238, 0, 0, 0, 0, 73,
    69, 78, 68, 174, 66, 96, 130,
];

const IMG_UNENC_HASH: &str = "afb3b949223b584fdd20bcdc301f59c66246414085aba6e2d9bc9fe7a0c15dc9";

const KEY: &[u8] = &[
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
];

#[test]
fn test_decrypt() {
    let mut file;
    unsafe {
        file = RpgFile::from_parts(
            IMG_ENC.to_vec(),
            crate::rpg_file::RpgFileType::Image,
            PathBuf::from("test_images/test.rpgmvp"),
        );
    }

    file.decrypt(KEY).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&file.data);
    let result = hasher.finalize();

    println!("\ndecrypted len: {}", file.data.len());
    assert_eq!(format!("{:x}", result), IMG_UNENC_HASH);
}

#[test]
fn test_decrypt_short() {
    let mut file;
    unsafe {
        file = RpgFile::from_parts(
            IMG_ENC[0..32].to_vec(),
            crate::rpg_file::RpgFileType::Image,
            PathBuf::from("test_images/test.rpgmvp"),
        );
    }

    let res = file.decrypt(KEY);
    assert!(matches!(res, Err(crate::error::Error::FileTooShort(_))));
}

#[test]
fn test_decryption_fail() {
    let mut file;
    unsafe {
        file = RpgFile::from_parts(
            IMG_ENC.to_vec(),
            crate::rpg_file::RpgFileType::Image,
            PathBuf::from("test_images/test.rpgmvp"),
        );
    }

    file.decrypt(&[1, 2, 3, 4, 5]).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&file.data);
    let result = hasher.finalize();

    assert_ne!(format!("{:x}", result), IMG_UNENC_HASH);
}

#[test]
fn test_create_path_from_output_flatten_1() {
    // Case 1
    let file1 = unsafe {
        RpgFile::from_parts(
            vec![],
            RpgFileType::Image,
            PathBuf::from("test_files/game/www/img/test.rpgmvp"),
        )
    };
    let out1 = OutputSettings::Flatten {
        dir: "output_dir".into(),
    };
    let gamepath1 = Path::new("test_files/game");

    let new_path = create_path_from_output(&out1, &file1, gamepath1).unwrap();

    assert_eq!(new_path, PathBuf::from("output_dir/www_img_test.png"));
}

#[test]
fn test_create_path_from_output_flatten_2() {
    let file1 = unsafe {
        RpgFile::from_parts(
            vec![],
            RpgFileType::Audio,
            PathBuf::from("../../game/www/img/test.rpgmvo"),
        )
    };
    let out1 = OutputSettings::Flatten {
        dir: "output_dir".into(),
    };
    let gamepath1 = Path::new("../../game");

    let new_path = create_path_from_output(&out1, &file1, gamepath1).unwrap();

    assert_eq!(new_path, PathBuf::from("output_dir/www_img_test.ogg"));
}

#[test]
fn test_create_path_from_output_replace_1() {
    let tmp_dir = TempDir::new("rrd-test").unwrap();

    let orig_file = tmp_dir.path().join("files/game/www/img/test.rpgmvo");
    fs::create_dir_all(&orig_file.parent().unwrap()).unwrap();
    fs::write(&orig_file, "test").unwrap();

    let file1 = unsafe { RpgFile::from_parts(vec![], RpgFileType::Audio, orig_file) };

    let out1 = OutputSettings::Replace;

    let gamepath1 = tmp_dir.path().join("files/game");

    let new_path = create_path_from_output(&out1, &file1, &gamepath1).unwrap();

    assert_eq!(new_path, tmp_dir.path().join("files/game/www/img/test.ogg"));
}

/*
/// Requires that a test game is present at ../test_files/test_game!
//#[test]
fn test_all() {
    let _ = Command::new("cp")
        .arg("-r")
        .arg("../test_files/test_game")
        .arg("../test_files/test_game_test")
        .spawn()
        .expect("failed to run cp -r")
        .wait();

    let game = RpgGame::new("../test_files/test_game_test", true);

    if let Ok(mut game) = game {
        let num_dec = game.decrypt_all(&crate::OutputSettings::Replace);

        if let Ok(num_dec) = num_dec {
            assert!(num_dec > 0);
        }
    }

    let _ = Command::new("trash")
        .arg("-r")
        .arg("../test_files/test_game_test")
        .spawn()
        .expect("failed to run rm -r")
        .wait();
}
*/
