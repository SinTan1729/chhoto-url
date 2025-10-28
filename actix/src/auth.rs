// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use actix_web::HttpRequest;
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use log::{debug, warn};
use passwords::PasswordGenerator;
use serde::Serialize;
use std::{rc::Rc, time::SystemTime};

use crate::config::Config;

// Define JSON struct for error response
#[derive(Serialize)]
pub struct APIResponse {
    pub success: bool,
    pub error: bool,
    reason: String,
    pass: bool,
}

// If the api_key environment variable exists
pub fn is_api_ok(http: HttpRequest, config: &Config) -> APIResponse {
    // If the api_key environment variable exists
    if config.api_key.is_some() {
        // If the header exists
        if let Some(header) = get_api_header(&http) {
            // If the header is correct
            if is_key_valid(header, config) {
                APIResponse {
                    success: true,
                    error: false,
                    reason: "Correct API key".to_string(),
                    pass: false,
                }
            } else {
                APIResponse {
                    success: false,
                    error: true,
                    reason: "Incorrect API key".to_string(),
                    pass: false,
                }
            }
        // The header may not exist when the user logs in through the web interface, so allow a request with no header.
        // Further authentication checks will be conducted in services.rs
        } else {
            // Due to the implementation of this result in services.rs, this JSON object will not be outputted.
            APIResponse {
                success: false,
                error: false,
                reason: "No valid authentication was found".to_string(),
                pass: true,
            }
        }
    } else {
        // If the API key isn't set, but an API Key header is provided
        if get_api_header(&http).is_some() {
            APIResponse {
                success: false,
                error: true,
                reason: "An API key was provided, but the 'api_key' environment variable is not configured in the Chhoto URL instance".to_string(), 
                pass: false
            }
        } else {
            APIResponse {
                success: false,
                error: false,
                reason: "".to_string(),
                pass: true,
            }
        }
    }
}
// Validate API key
pub fn is_key_valid(key: &str, config: &Config) -> bool {
    if let Some(api_key) = &config.api_key {
        // Check if API Key is hashed using Argon2. More algorithms maybe added later.
        let authorized = if config.hash_algorithm.is_some() {
            debug!("Using Argon2 hash for API key validation.");
            let hash = PasswordHash::new(api_key).expect("The provided password hash is invalid.");
            Argon2::default()
                .verify_password(key.as_bytes(), &hash)
                .is_ok()
        } else {
            // If hashing is not enabled, use the plaintext API key for matching
            api_key == key
        };
        if !authorized {
            warn!("Incorrect API key was provided when connecting to Chhoto URL.");
            false
        } else {
            debug!("Server accessed with API key.");
            true
        }
    } else {
        warn!("API was accessed with API key validation but no API key was specified. Set the 'api_key' environment variable.");
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
pub fn get_api_header(req: &HttpRequest) -> Option<&str> {
    req.headers().get("X-API-Key")?.to_str().ok()
}

// Validate a session
pub fn is_session_valid(session: Session, config: &Config) -> bool {
    // If there's no password provided, just return true
    if config.password.is_none() {
        return true;
    }

    if let Ok(token) = session.get::<String>("chhoto-url-auth") {
        is_token_valid(token.as_deref())
    } else {
        false
    }
}

// Check a token cryptographically
fn is_token_valid(token: Option<&str>) -> bool {
    if let Some(token_body) = token {
        let token_parts: Rc<[&str]> = token_body.split(';').collect();
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
