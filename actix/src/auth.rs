// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::Session;
use std::{env, time::SystemTime};

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
