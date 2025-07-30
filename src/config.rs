use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
        
        let contents = fs::read_to_string(&path)
            .context("Failed to read config file")?;
        
        toml::from_str(&contents)
            .context("Failed to parse config file")
    }
    
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&path, contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }
    
    pub fn set_oauth_token(&mut self, token: OAuthToken) {
        self.oauth_token = Some(token);
    }
    
    fn config_path() -> Result<PathBuf> {
        let home = home_dir()
            .context("Failed to find home directory")?;
        
        Ok(home.join(".config").join("linear-tui").join("config.toml"))
    }
}