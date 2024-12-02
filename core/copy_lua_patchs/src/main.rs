use std::{
    env::var,
    fs::{self, read_dir},
};

fn main() {
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
