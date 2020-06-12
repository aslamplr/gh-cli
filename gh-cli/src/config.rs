use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

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

pub(crate) fn get_config() -> Option<Config> {
    let path = get_config_path()?;
    if path.exists() {
        fs::read(path)
            .map(|c| toml::from_slice::<Config>(c.as_slice()).ok())
            .ok()
            .flatten()
    } else {
        None
    }
}

pub(crate) fn save_config(config: Config) -> Result<PathBuf> {
    let err_fn = || anyhow::anyhow!("Couldn't establish a config path!");
    let path = get_config_path().ok_or_else(err_fn)?;
    if !path.exists() {
        let parent = path.parent().ok_or_else(err_fn)?;
        std::fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string(&config)?;
    let mut file = File::create(&path)?;
    write!(file, "{}", toml)?;
    Ok(path)
}
