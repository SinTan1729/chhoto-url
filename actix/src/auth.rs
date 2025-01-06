// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::HttpRequest;
use std::{env, time::SystemTime};

// API key generation and scoring
use passwords::{analyzer, scorer, PasswordGenerator};

// Validate API key
pub fn validate_key(key: String) -> bool {
    if let Ok(api_key) = env::var("api_key") {
        if api_key != key {
            eprintln!("Incorrect API key was provided when connecting to Chhoto URL.");
            false
        } else {
            eprintln!("Server accessed with API key.");
            true
        }
    } else {
        eprintln!("API was accessed with API key validation but no API key was specified. Set the 'api_key' environment variable.");
        false
    }
}

// Generate an API key if the user doesn't specify a secure key
// Called in main.rs
pub fn gen_key() -> String {
    let key = PasswordGenerator {
        length: 128,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        spaces: false,
        exclude_similar_characters: false,
        strict: true,
    };
    key.generate_one().unwrap()
}

// Check if the API key header exists
pub fn api_header(req: &HttpRequest) -> Option<&str> {
    req.headers().get("X-API-Key")?.to_str().ok()
}

// Determine whether the inputted API key is sufficiently secure
pub fn is_key_secure() -> bool {
    let score = scorer::score(&analyzer::analyze(env::var("api_key").unwrap()));
    score >= 90.0
}

// Validate a given password
pub fn validate(session: Session) -> bool {
    // If there's no password provided, just return true
    if env::var("password")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return true;
    }

    if let Ok(token) = session.get::<String>("chhoto-url-auth") {
        check(token)
    } else {
        false
    }
}

// Check a token cryptographically
fn check(token: Option<String>) -> bool {
    if let Some(token_body) = token {
        let token_parts: Vec<&str> = token_body.split(';').collect();
        if token_parts.len() < 2 {
            false
        } else {
            let token_text = token_parts[0];
            let token_time = token_parts[1].parse::<u64>().unwrap_or(0);
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            token_text == "chhoto-url-auth" && time_now < token_time + 1209600 // There are 1209600 seconds in 14 days
        }
    } else {
        false
    }
}

// Generate a new cryptographic token
pub fn gen_token() -> String {
    let token_text = String::from("chhoto-url-auth");
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_secs();
    format!("{token_text};{time}")
}
