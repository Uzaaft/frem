use anyhow::Result;
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

    println!("Choose auth method:");
    println!("1) OAuth");
    println!("2) API Key");
    print!("> ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "1" => {
            let token = oauth::authenticate()?;
            let client = LinearClient::new_with_oauth(token.access_token.clone())?;

            match client.get_viewer() {
                Ok(user) => {
                    println!("Authenticated as {}", user.name);
                    config.oauth_token = Some(token);
                    config.save()?;
                    Ok(client)
                }
                Err(e) => Err(e.into()),
            }
        }
        "2" => {
            println!("Enter Linear API key:");
            print!("> ");
            io::stdout().flush()?;

            let mut api_key = String::new();
            io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();

            let client = LinearClient::new(api_key.clone())?;

            match client.get_viewer() {
                Ok(user) => {
                    println!("Authenticated as {}", user.name);
                    config.api_key = Some(api_key);
                    config.save()?;
                    Ok(client)
                }
                Err(e) => Err(e.into()),
            }
        }
        _ => anyhow::bail!("Invalid choice"),
    }
}
