use std::{
    env::var,
    fs::{self, copy, read_dir},
};

fn main() {
    println!("cargo:rerun-if-changed=NULL");

    #[cfg(target_os = "windows")]
    forward_dll::forward_dll("C:\\Windows\\System32\\winmm.dll").unwrap();

    let balatro_game_path = var("BALATRO_GAME_PATH").unwrap();
    copy(
        "target/debug/winmm.dll",
        format!("{}\\winmm.dll", balatro_game_path),
    )
    .unwrap();

    let paths = read_dir("../patchs").unwrap();
    let appdata = var("APPDATA").unwrap();

    fs::create_dir_all(format!("{}\\Balatro\\Mods\\balatro-vs\\lovely", appdata)).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let file_name = path.file_name().unwrap();
        let file_name = file_name.to_str().unwrap();
        let dest = format!(
            "{}\\Balatro\\Mods\\balatro-vs\\lovely\\{}",
            appdata, file_name
        );

        fs::copy(path, dest).unwrap();
    }
}
