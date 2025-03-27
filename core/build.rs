use std::{
    env::var,
    fs::{self, copy, read_dir, ReadDir},
    path::Path,
};

fn main() {
    println!("cargo:rerun-if-changed=NULL");

    #[cfg(target_os = "windows")]
    forward_dll::forward_dll("C:\\Windows\\System32\\winmm.dll").unwrap();

    let paths = read_dir("../patchs").unwrap();
    let profile = var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    copy_patches(
        paths,
        "balatro-vs\\lovely".to_string(),
        format!("target\\{}\\", profile),
    );

    if profile == "release" {
        let bvs_config_path =
            Path::new(&format!("target\\{}\\balatro-vs\\lovely", profile)).join("bvs.json");

        if bvs_config_path.exists() {
            update_bvs_json(&bvs_config_path, "wss", "live.balatro-vs-matchmaking.eu", 0);
        }
    }

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

fn update_bvs_json(path: &Path, protocol: &str, host: &str, port: u16) {
    let content = fs::read_to_string(path).expect("Failed to read bvs.json");

    // protocol
    let content = content.replace(
        r#""protocol": "ws""#,
        &format!(r#""protocol": "{}""#, protocol),
    );

    // host
    let content = content.replace(r#""host": "localhost""#, &format!(r#""host": "{}""#, host));

    // port
    let content = content.replace(r#""port": 3536"#, &format!(r#""port": {}"#, port));

    fs::write(path, content).expect("Failed to write updated bvs.json");
}
