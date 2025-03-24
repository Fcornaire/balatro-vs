use std::{
    env::var,
    fs::{self, copy, read_dir, ReadDir},
};

fn main() {
    println!("cargo:rerun-if-changed=NULL");

    #[cfg(target_os = "windows")]
    forward_dll::forward_dll("C:\\Windows\\System32\\winmm.dll").unwrap();

    let paths = read_dir("../patchs").unwrap();

    copy_patches(
        paths,
        "balatro-vs\\lovely".to_string(),
        format!("target\\{}\\", var("PROFILE").unwrap()),
    );

    if let Some(balatro_game_path) = var("BALATRO_GAME_PATH").ok() {
        copy(
            "target/debug/winmm.dll",
            format!("{}\\winmm.dll", balatro_game_path),
        )
        .unwrap();

        let appdata = var("APPDATA").unwrap();

        let paths = read_dir("../patchs").unwrap();

        copy_patches(
            paths,
            "\\Balatro\\Mods\\balatro-vs\\lovely".to_string(),
            appdata,
        );
    }
}

fn copy_patches(to_copy: ReadDir, target_path: String, target_dir: String) {
    fs::create_dir_all(format!("{}{}", target_dir, target_path)).unwrap();

    for path in to_copy {
        let path = path.unwrap().path();
        let file_name = path.file_name().unwrap();
        let file_name = file_name.to_str().unwrap();
        let dest = format!("{}{}\\{}", target_dir, target_path, file_name);

        fs::copy(path, dest).unwrap();
    }
}
