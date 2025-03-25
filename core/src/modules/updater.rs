use std::{fs, io::Write, path::Path, process::Command};

use reqwest::Error;
use semver::Version;
use serde::Deserialize;
use tracing::{error, info};
use zip::ZipArchive;

use crate::get_bvs_config;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Deserialize, Debug)]
struct Tag {
    name: String,
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

    pub async fn trigger_update(base_download_url: &str, last_version: &str) -> bool {
        info!("[Updater] Triggering update");

        let client = reqwest::Client::new();
        let download_url = format!(
            "{}/releases/download/{}/balatro-vs.zip",
            base_download_url, last_version
        );

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
            return false; //TODO: re test later
        }

        let content = res.bytes().await.unwrap();
        let mut file = File::create(download_path).await.unwrap();
        file.write_all(&content).await.unwrap();

        info!("[Updater] Downloaded latest version ");
        true
    }

    pub fn update(&self) {
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
        let target_path = std::env::current_dir().unwrap().join("winmm.dll");

        let current_pid = std::process::id();

        #[cfg(target_os = "windows")]
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

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let script_content = format!(
            r#"
                #!/bin/bash
                while kill -0 {pid} 2>/dev/null; do
                    sleep 5
                done
                cp "{src}" "{dst}"
                if [ $? -ne 0 ]; then
                    echo "Failed to copy file."
                    exit 1
                fi
                rm -rf "{extract_path}"
                rm -- "$0"
            "#,
            pid = current_pid,
            src = main_dll_path.display(),
            dst = target_path.display(),
            extract_path = extract_path.display()
        );

        let script_path = std::env::current_dir()
            .unwrap()
            .join(if cfg!(target_os = "windows") {
                "update_and_cleanup.bat"
            } else {
                "update_and_cleanup.sh"
            });
        let mut file = std::fs::File::create(&script_path).unwrap();
        file.write_all(script_content.as_bytes()).unwrap();

        #[cfg(target_os = "linux")]
        Command::new("chmod")
            .arg("+x")
            .arg(&script_path)
            .output()
            .expect("Failed to make script executable");

        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", script_path.to_str().unwrap()])
                .spawn()
                .expect("Failed to start batch script");
        } else {
            Command::new("sh")
                .arg(script_path.to_str().unwrap())
                .spawn()
                .expect("Failed to start shell script");
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    pub fn update_current_version(&mut self) {
        self.current_version = get_bvs_config().clone().get_version();
    }

    pub async fn get_last_stable_version(repository_url: &str) -> Result<String, Error> {
        let tags_url = format!("{}/tags", repository_url);
        let client = reqwest::Client::new();
        let res = client
            .get(&tags_url)
            .header("User-Agent", "balatro-vs")
            .send()
            .await?
            .json::<Vec<Tag>>()
            .await?;

        let mut tags = res
            .iter()
            .map(|tag| tag.name.clone())
            .collect::<Vec<String>>();
        tags.sort_by(|a, b| {
            Version::parse(b)
                .unwrap_or_else(|_| Version::new(0, 0, 0))
                .cmp(&Version::parse(a).unwrap_or_else(|_| Version::new(0, 0, 0)))
        });
        let default_version = "0.0.0".to_string();
        let latest_tag = tags.first().unwrap_or(&default_version);

        Ok(latest_tag.clone())
    }
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
