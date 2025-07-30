use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub oauth_token: Option<OAuthToken>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            api_key: None,
            oauth_token: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }
    
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
    
    
    fn config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("No config dir"))?
            .join("linear-tui")
            .join("config.toml"))
    }
}