use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::api::client::LinearClient;
use crate::config::Config;
use crate::oauth;

pub fn ensure_authenticated() -> Result<LinearClient> {
    let mut config = Config::load()?;
    
    if let Some(token) = &config.oauth_token {
        return LinearClient::new_with_oauth(token.access_token.clone());
    }
    
    if let Some(api_key) = &config.api_key {
        return LinearClient::new(api_key.clone());
    }
    
    println!("No authentication found. Choose authentication method:");
    println!("1) OAuth (recommended)");
    println!("2) API Key");
    print!("> ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => {
            let token = oauth::authenticate()?;
            let client = LinearClient::new_with_oauth(token.access_token.clone())?;
            
            print!("Verifying authentication... ");
            io::stdout().flush()?;
            
            match client.get_viewer() {
                Ok(user) => {
                    println!("Success! Authenticated as {}", user.name);
                    config.set_oauth_token(token);
                    config.save()?;
                    Ok(client)
                }
                Err(e) => {
                    println!("Failed!");
                    Err(e).context("OAuth authentication failed")
                }
            }
        }
        "2" => {
            println!("Please enter your Linear API key:");
            println!("You can get one from: https://linear.app/settings/api");
            print!("> ");
            io::stdout().flush()?;
            
            let mut api_key = String::new();
            io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();
            
            let client = LinearClient::new(api_key.clone())?;
            
            print!("Verifying API key... ");
            io::stdout().flush()?;
            
            match client.get_viewer() {
                Ok(user) => {
                    println!("Success! Authenticated as {}", user.name);
                    config.set_api_key(api_key);
                    config.save()?;
                    Ok(client)
                }
                Err(e) => {
                    println!("Failed!");
                    Err(e).context("Invalid API key")
                }
            }
        }
        _ => anyhow::bail!("Invalid choice"),
    }
}