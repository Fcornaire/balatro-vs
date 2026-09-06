use std::{fs, path::Path};

use semver::Version;
use serde::Deserialize;
use tracing::{error, info};
use zip::ZipArchive;

use crate::get_bvs_config;

#[derive(Deserialize, Debug)]
struct Tag {
    name: String,
}

#[cfg(target_os = "switch")]
pub mod nx {
    use std::sync::OnceLock;

    pub type HttpGet = fn(&str) -> Result<Vec<u8>, String>;

    static HTTP_GET: OnceLock<HttpGet> = OnceLock::new();

    pub const SD_ROOT: &str = "sd:/";
    pub const ZIP_PATH: &str = "sd:/Balatro/Mods/balatro-vs-switch-update.zip";
    pub const NO_UPDATE_MARKER: &str = "sd:/Balatro/Mods/balatro-vs/no_update";

    pub fn set_http_get(func: HttpGet) {
        let _ = HTTP_GET.set(func);
    }

    pub fn http_get(url: &str) -> Result<Vec<u8>, String> {
        let get = HTTP_GET.get().ok_or("no HTTP client registered")?;
        get(url)
    }

    /// Runs `func` on a worker thread that never exits
    /// On the Switch a thread with TLS destructors crashes in `nn::ro` on exit
    pub fn spawn(func: impl FnOnce() + Send + 'static) {
        let _ = std::thread::Builder::new()
            .name("bvs-updater".to_string())
            .spawn(move || {
                crate::run_thread_start_hook();
                func();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                }
            });
    }
}

#[derive(Debug, Clone)]
pub struct Updater {
    current_version: String,
    last_version: String,
    repository_url: String,
    base_download_url: String,
    is_updating: bool,
    should_update: bool,
}

impl Updater {
    pub fn new() -> Self {
        Self {
            current_version: "0.0.0".to_string(),
            last_version: "0.0.0".to_string(),
            repository_url: "https://api.github.com/repos/fcornaire/balatro-vs".to_string(),
            base_download_url: "https://github.com/fcornaire/balatro-vs".to_string(),
            is_updating: false,
            should_update: false,
        }
    }

    pub fn get_repository_url(&self) -> &str {
        &self.repository_url
    }

    pub fn get_base_download_url(&self) -> &str {
        &self.base_download_url
    }

    pub fn set_current_version(&mut self, version: String) {
        self.current_version = version;
    }

    pub fn set_should_update(&mut self, should_update: bool) {
        self.should_update = should_update;
    }

    pub fn should_update(&self) -> bool {
        self.should_update
    }

    pub fn set_last_version(&mut self, version: String) {
        self.last_version = version;
    }

    pub fn get_current_version(&self) -> String {
        self.current_version.clone()
    }

    pub fn get_last_version(&self) -> String {
        self.last_version.clone()
    }

    pub fn is_updating(&self) -> bool {
        self.is_updating
    }

    pub fn set_is_updating(&mut self, is_updating: bool) {
        self.is_updating = is_updating;
    }

    pub fn is_thunderstore_build() -> bool {
        cfg!(feature = "thunderstore_build")
    }

    pub fn update_current_version(&mut self) {
        self.current_version = get_bvs_config().clone().get_version();
    }

    #[cfg(not(target_os = "switch"))]
    pub async fn trigger_update(base_download_url: &str, last_version: &str) -> bool {
        use tokio::io::AsyncWriteExt;

        info!("[Updater] Triggering update");

        let client = reqwest::Client::new();

        #[cfg(target_os = "android")]
        let download_url = format!(
            "{}/releases/download/{}/balatro-vs-{}-android.zip",
            base_download_url, last_version, last_version
        );

        #[cfg(not(target_os = "android"))]
        let download_url = format!(
            "{}/releases/download/{}/balatro-vs.zip",
            base_download_url, last_version
        );

        #[cfg(target_os = "android")]
        let download_path = std::env::current_dir()
            .unwrap()
            .join("balatro-vs-android.zip");

        #[cfg(not(target_os = "android"))]
        let download_path = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap())
            .join("balatro-vs.zip");

        let res = client
            .get(&download_url)
            .header("User-Agent", "balatro-vs")
            .send()
            .await
            .unwrap();

        if res.status().is_client_error() || res.status().is_server_error() {
            error!(
                "[Updater] Failed to download latest version: {:?}",
                res.text().await
            );
            return false;
        }

        let content = res.bytes().await.unwrap();
        let mut file = tokio::fs::File::create(download_path).await.unwrap();
        file.write_all(&content).await.unwrap();

        info!("[Updater] Downloaded latest version ");
        true
    }

    /// Switch downloads update to the SD card.
    #[cfg(target_os = "switch")]
    pub fn trigger_update_blocking(base_download_url: &str, last_version: &str) -> bool {
        info!("[Updater] Triggering update");
        let url = format!(
            "{}/releases/download/{}/balatro-vs-{}-switch.zip",
            base_download_url, last_version, last_version
        );
        match nx::http_get(&url)
            .and_then(|bytes| fs::write(nx::ZIP_PATH, bytes).map_err(|e| e.to_string()))
        {
            Ok(()) => {
                info!("[Updater] Downloaded latest version to {}", nx::ZIP_PATH);
                true
            }
            Err(e) => {
                error!("[Updater] Failed to download latest version: {e}");
                false
            }
        }
    }

    pub fn update(&self) {
        #[cfg(target_os = "android")]
        {
            self.update_android();
            return;
        }

        #[cfg(target_os = "switch")]
        {
            Self::apply_switch_update();
            return;
        }

        #[cfg(target_os = "windows")]
        {
            use std::io::Write;

            //unzip the downloaded zip
            let download_path = dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap())
                .join("balatro-vs.zip");
            let extract_path = download_path.parent().unwrap().join("balatro-vs-update");
            let res = unzip_file(&download_path.clone(), &extract_path.clone());

            if res.is_err() {
                error!("[Updater] Failed to unzip file {:?}", res.err());
                return;
            }

            let extract_path = dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap())
                .join("balatro-vs-update");
            let main_dll_path = extract_path.join("release").join("winmm.dll");
            let lovely_patch_path = extract_path.join("release").join("balatro-vs");

            //copy the lovely patch to the mods folder
            let lovely_mod_folder = dirs::data_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap())
                .join("Balatro")
                .join("Mods")
                .join("balatro-vs");

            let res = copy_directory(&lovely_patch_path, &lovely_mod_folder);
            if let Err(e) = res {
                error!("[Updater] Failed to copy the lovely patch: {:?}", e);
            }

            //copy the main dll to the game folder
            let game_dir = resolve_game_directory();
            let target_path = game_dir.join("winmm.dll");
            let current_pid = std::process::id();

            let script_content = format!(
                r#"
                    @echo off
                    :wait_loop
                    tasklist /FI "PID eq {pid}" 2>NUL | find /I /N "{pid}">NUL
                    if "%ERRORLEVEL%"=="0" (
                        timeout /T 5 >NUL
                        goto wait_loop
                    )
                    copy "{src}" "{dst}"
                    if %errorlevel% neq 0 (
                        echo Failed to copy file.
                        exit /b %errorlevel%
                    )
                    rmdir /S /Q "{extract_path}"
                    del "%~f0"
                "#,
                pid = current_pid,
                src = main_dll_path.display(),
                dst = target_path.display(),
                extract_path = extract_path.display()
            );

            let script_path = game_dir.join("update_and_cleanup.bat");
            let mut file = std::fs::File::create(&script_path).unwrap();
            file.write_all(script_content.as_bytes()).unwrap();

            std::process::Command::new("cmd")
                .args(&["/C", script_path.to_str().unwrap()])
                .spawn()
                .expect("Failed to start batch script");

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    #[cfg(target_os = "android")]
    fn update_android(&self) {
        crate::lua_print("[BVS] starting update");

        let zip_path = std::env::current_dir()
            .unwrap()
            .join("balatro-vs-android.zip");

        if !zip_path.exists() {
            crate::lua_print(&format!("[BVS] android zip not found at {:?}", zip_path));
            return;
        }

        let mods_dir = std::env::current_dir().unwrap().join("Mods");
        crate::lua_print(&format!("[BVS] Extracting to {:?}", mods_dir));

        let res = unzip_file_android(&zip_path, &mods_dir);
        if let Err(e) = res {
            crate::lua_print(&format!("[BVS] Extraction failed: {:?}", e));
            return;
        }

        let _ = std::fs::remove_file(&zip_path);
        crate::lua_print("[BVS] Android update applied successfully");
    }

    #[cfg(target_os = "switch")]
    pub fn apply_switch_update() -> bool {
        let zip_path = Path::new(nx::ZIP_PATH);
        if !zip_path.exists() {
            return false;
        }
        match unzip_file(zip_path, Path::new(nx::SD_ROOT)) {
            Ok(()) => {
                let _ = fs::remove_file(zip_path);
                info!("[Updater] Switch update applied, restart the game");
                true
            }
            Err(e) => {
                error!("[Updater] Extraction failed: {e:?}");
                false
            }
        }
    }

    #[cfg(not(target_os = "switch"))]
    pub async fn get_last_stable_version(repository_url: &str) -> Result<String, reqwest::Error> {
        let tags_url = format!("{}/tags", repository_url);
        let client = reqwest::Client::new();
        let res = client
            .get(&tags_url)
            .header("User-Agent", "balatro-vs")
            .send()
            .await?
            .json::<Vec<Tag>>()
            .await?;
        Ok(latest_tag(res))
    }

    #[cfg(target_os = "switch")]
    pub fn get_last_stable_version_blocking(repository_url: &str) -> Result<String, String> {
        let body = nx::http_get(&format!("{}/tags", repository_url))?;
        let tags: Vec<Tag> = serde_json::from_slice(&body).map_err(|e| format!("tags: {e}"))?;
        Ok(latest_tag(tags))
    }
}

fn latest_tag(tags: Vec<Tag>) -> String {
    let mut names: Vec<String> = tags.into_iter().map(|t| t.name).collect();
    names.sort_by(|a, b| {
        Version::parse(b)
            .unwrap_or_else(|_| Version::new(0, 0, 0))
            .cmp(&Version::parse(a).unwrap_or_else(|_| Version::new(0, 0, 0)))
    });
    names
        .into_iter()
        .next()
        .unwrap_or_else(|| "0.0.0".to_string())
}

fn unzip_file(zip_path: &Path, extract_to: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = extract_to.join(file.name());

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

/// For Android, we need to not overwrite the existing .so at runtime, it will be swapped next game launch
#[cfg(target_os = "android")]
fn unzip_file_android(
    zip_path: &Path,
    extract_to: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        let dest_name = if name.ends_with(".so") {
            format!("{}.new", name)
        } else {
            name.clone()
        };

        let outpath = extract_to.join(&dest_name);

        if name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn copy_directory(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_directory(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
            info!("[Updater] Copied {:?} to {:?}", entry_path, dest_path);
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn resolve_game_directory() -> std::path::PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        return cwd;
    }

    std::env::temp_dir()
}
