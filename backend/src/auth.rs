// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_session::{Session, SessionExt};
use actix_web::cookie::Key;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, web};
use argon2::PasswordVerifier;
use log::{debug, warn};
use std::pin::Pin;
use std::time::SystemTime;

use crate::config::Credential::{self, Argon2, Plain};
use crate::{AppState, services::types::JSONResponse};

// Read API key from header and process it
async fn is_api_ok(req: HttpRequest, api_key: Option<Credential>) -> JSONResponse {
    let api_header = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok());
    // If the api_key environment variable exists
    if let Some(key) = api_key {
        // If the header exists
        if let Some(header) = api_header {
            // If the header is correct
            match is_key_valid(header.to_string(), key).await {
                Ok(()) => JSONResponse {
                    success: true,
                    error: false,
                    reason: "Correct API key.".to_owned(),
                },
                Err(e) => JSONResponse {
                    success: false,
                    error: true,
                    reason: e,
                },
            }
        // The header may not exist when the user logs in through the web interface, so allow a request with no header.
        // Further authentication checks will be conducted in services.rs
        } else {
            // Due to the implementation of this result in services.rs, this JSON object will not be outputted.
            JSONResponse {
                success: false,
                error: false,
                reason: "No valid authentication.".to_owned(),
            }
        }
    } else {
        // If the API key isn't set, but an API Key header is provided
        if api_header.is_some() {
            JSONResponse {
                success: false,
                error: true,
                reason: "API key authentication is disabled.".to_owned(),
            }
        } else {
            JSONResponse {
                success: false,
                error: false,
                reason: "No valid authentication.".to_owned(),
            }
        }
    }
}

// Validate API key
async fn is_key_valid(key: String, api_key: Credential) -> Result<(), String> {
    // Check if API Key is hashed using Argon2. More algorithms maybe added later.
    let authorized = match api_key {
        Argon2(hash) => actix_web::web::block(move || {
            argon2::Argon2::default().verify_password(key.as_bytes(), hash.as_ref())
        })
        .await
        .map_err(|_| "API key validation temporarily unavailable.".to_string())?
        .is_ok(),
        Plain(val) => {
            // If hashing is not enabled, use the plaintext API key for matching
            subtle::ConstantTimeEq::ct_eq(val.as_bytes(), key.as_bytes()).into()
        }
    };
    if !authorized {
        warn!("Incorrect API key was provided.");
        Err("Wrong API key.".to_string())
    } else {
        debug!("Server accessed with API key.");
        Ok(())
    }
}

// Generate an API key if the user doesn't specify a secure key
pub(crate) fn gen_key() -> String {
    base64_url::encode(Key::generate().master())
}

// Validate a session
fn is_session_valid(session: &Session) -> bool {
    if let Ok(token) = session.get::<String>("chhoto-url-auth") {
        is_token_valid(token.as_deref())
    } else {
        false
    }
}

// Check a token cryptographically
fn is_token_valid(token: Option<&str>) -> bool {
    if let Some(token_body) = token {
        let token_parts = token_body.split_once(';');
        if let Some((marker, expiry)) = token_parts {
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            (marker == "chhoto-url-password" || marker == "chhoto-url-oidc")
                && time_now < expiry.parse::<u64>().unwrap_or(0)
        } else {
            false
        }
    } else {
        false
    }
}

// Enum for auth state
pub(crate) enum Auth {
    ValidAPIKey,
    ValidSession,
    NoAuthNeeded,
    None { result: JSONResponse },
    InvalidAPIKey { result: JSONResponse },
}
// Extractor for authentication
impl FromRequest for Auth {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Auth, actix_web::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        let data = req
            .app_data::<web::Data<AppState>>()
            .expect("Appstate wasn't created yet. THIS SHOULD NEVER OCCUR!!!");
        let api_key = data.config.api_key.clone();
        let no_password = data.config.password.is_none();
        let no_oidc = data.oidc_data.is_none();

        Box::pin(async move {
            let api_result = is_api_ok(req.clone(), api_key).await;

            if api_result.success {
                return Ok(Auth::ValidAPIKey);
            }

            if api_result.error {
                return Ok(Auth::InvalidAPIKey { result: api_result });
            }

            if no_password && no_oidc {
                return Ok(Auth::NoAuthNeeded);
            }

            let session = req.get_session();

            if is_session_valid(&session) {
                return Ok(Auth::ValidSession);
            }

            Ok(Auth::None { result: api_result })
        })
    }
}

// Generate a new token for usage in cookie
pub(crate) fn gen_token_text(remember: bool, oidc: bool) -> String {
    let token_text = if oidc {
        "chhoto-url-oidc"
    } else {
        "chhoto-url-password"
    };

    let remember_sec = if remember {
        604800 // 7 days
    } else {
        900 // 15 minutes
    };
    let time = (SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards!")
        .as_secs()
        + remember_sec)
        .to_string(); // Valid for 7 days
    let mut out = String::with_capacity(token_text.len() + time.len() + 1);
    out.push_str(token_text);
    out.push(';');
    out.push_str(&time);
    out
}
