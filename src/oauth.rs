use anyhow::Result;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, thread_rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};
use url::Url;

use crate::config::OAuthToken;

const CLIENT_ID: &str = "linear-tui";
const REDIRECT_URI: &str = "http://localhost:8989/callback";
const AUTH_URL: &str = "https://linear.app/oauth/authorize";
const TOKEN_URL: &str = "https://api.linear.app/oauth/token";

#[derive(Debug, Serialize)]
struct TokenRequest {
    code: String,
    redirect_uri: String,
    client_id: String,
    code_verifier: String,
    grant_type: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

pub fn authenticate() -> Result<OAuthToken> {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    let auth_url = build_auth_url(&code_challenge, &state)?;

    println!("Opening browser...\n{}", auth_url);

    open::that(auth_url.as_str()).ok();

    let (tx, rx) = mpsc::channel();
    let state_clone = state.clone();

    thread::spawn(move || {
        if let Err(e) = run_callback_server(tx, &state_clone) {
            eprintln!("Callback server error: {}", e);
        }
    });

    let code = rx.recv_timeout(Duration::from_secs(300))??;

    exchange_code_for_token(&code, &code_verifier)
}

fn generate_code_verifier() -> String {
    let mut verifier = vec![0u8; 64];
    thread_rng().fill_bytes(&mut verifier);
    URL_SAFE_NO_PAD.encode(&verifier)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

fn generate_state() -> String {
    let mut state = vec![0u8; 16];
    thread_rng().fill_bytes(&mut state);
    URL_SAFE_NO_PAD.encode(&state)
}

fn build_auth_url(code_challenge: &str, state: &str) -> Result<Url> {
    let mut url = Url::parse(AUTH_URL)?;
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", "read write issues:create comments:create")
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("prompt", "consent");
    Ok(url)
}

fn run_callback_server(tx: mpsc::Sender<Result<String>>, expected_state: &str) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8989")?;

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 2048];
        stream.read(&mut buffer)?;

        let request = String::from_utf8_lossy(&buffer);

        if let Some(line) = request.lines().next() {
            if line.starts_with("GET /callback") {
                let url = format!("http://localhost{}", &line[4..line.len() - 9]);
                let parsed = Url::parse(&url)?;
                let params: HashMap<_, _> = parsed.query_pairs().collect();

                if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
                    if state == expected_state {
                        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n<html><body>Success! You can close this window.</body></html>")?;
                        tx.send(Ok(code.to_string()))?;
                        return Ok(());
                    } else {
                        tx.send(Err(anyhow::anyhow!("Invalid state parameter")))?;
                    }
                } else if let Some(error) = params.get("error") {
                    tx.send(Err(anyhow::anyhow!("OAuth error: {}", error)))?;
                }
            }
        }
    }

    Ok(())
}

fn exchange_code_for_token(code: &str, code_verifier: &str) -> Result<OAuthToken> {
    let client = reqwest::blocking::Client::new();

    let token_request = TokenRequest {
        code: code.to_string(),
        redirect_uri: REDIRECT_URI.to_string(),
        client_id: CLIENT_ID.to_string(),
        code_verifier: code_verifier.to_string(),
        grant_type: "authorization_code".to_string(),
    };

    let response = client.post(TOKEN_URL).json(&token_request).send()?;

    if !response.status().is_success() {
        anyhow::bail!("Token exchange failed: {}", response.text()?);
    }

    let token_response: TokenResponse = response.json()?;

    Ok(OAuthToken {
        access_token: token_response.access_token,
        token_type: token_response.token_type,
        scope: token_response.scope,
    })
}
