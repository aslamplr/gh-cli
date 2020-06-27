#![cfg(feature = "config")]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::task;

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub user_name: String,
    pub access_token: String,
}

impl Config {
    pub fn new(user_name: &str, access_token: &str) -> Self {
        Self {
            user_name: user_name.to_owned(),
            access_token: access_token.to_owned(),
        }
    }
}

pub(crate) fn get_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|x| x.join(".config/gh-cli/config.toml"))
}

pub(crate) async fn get_config() -> Option<Config> {
    let path = get_config_path()?;
    if fs::metadata(&path).await.is_ok() {
        let content = fs::read(path).await.ok()?;
        task::spawn_blocking(move || toml::from_slice::<Config>(content.as_slice()).ok())
            .await
            .ok()
            .flatten()
    } else {
        None
    }
}

pub(crate) async fn save_config(config: Config) -> Result<PathBuf> {
    let err_fn = || anyhow::anyhow!("Couldn't establish a config path!");
    let path = get_config_path().ok_or_else(err_fn)?;
    if fs::metadata(&path).await.is_err() {
        let parent = path.parent().ok_or_else(err_fn)?;
        fs::create_dir_all(parent).await?;
    }
    let toml = task::spawn_blocking(move || toml::to_string(&config)).await??;
    let mut file = File::create(&path).await?;
    file.write_all(toml.as_bytes()).await?;
    Ok(path)
}
