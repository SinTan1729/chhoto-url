// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    Error, FromRequest, HttpRequest,
    cookie::{Cookie, CookieJar, Key, SameSite, time::Duration},
    dev::Payload,
    http::header::HeaderMap,
    web,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use constant_time_eq::constant_time_eq;
use log::{debug, info, warn};
use sha2::{Digest, Sha512};
use std::pin::Pin;
use std::{sync::Arc, time::SystemTime};
use tokio::{sync::Semaphore, task::spawn_blocking};

use crate::background::acquire_permit;
use crate::{AppState, services::types::JSONResponse};

const API_HEADER: &str = "X-API-Key";
pub(crate) const COOKIE_NAME: &str = "chhoto-url-auth";
const COOKIE_VALUE_PASSWORD: &str = "chhoto-url-password";
const COOKIE_VALUE_OIDC: &str = "chhoto-url-oidc";

#[derive(Clone)]
pub(crate) enum Credential {
    Plain(String),
    Argon2(Arc<argon2::PasswordHash>),
    SHA512(Arc<[u8; 64]>),
}
impl Credential {
    pub(crate) fn parse(val: String, hash_algorithm: &Option<String>, name: &str) -> Option<Self> {
        const EXIT_MSG: &str = "Chhoto URL exit due to bad auth!";
        let recommended = if name == "Password" {
            "Argon2"
        } else {
            "SHA512"
        };
        match hash_algorithm.as_deref() {
            Some("Argon2") => match PasswordHash::new(&val) {
                Ok(hash) => {
                    info!(
                        "Using Argon2 for {name} verification.{}",
                        if recommended == "Argon2" {
                            ""
                        } else {
                            " Argon2 is recommended."
                        }
                    );
                    Some(Credential::Argon2(hash.into()))
                }
                Err(e) => {
                    eprintln!("Invalid Argon2 hash was provided for {name}: {e}");
                    panic!("{EXIT_MSG}");
                }
            },
            Some("SHA512") => match base64_url::decode(&val) {
                Ok(hash) => {
                    info!(
                        "Using SHA512 for {name} verification.{}",
                        if recommended == "SHA512" {
                            ""
                        } else {
                            " SHA512 is recommended."
                        }
                    );
                    let Ok(hash): Result<[u8; 64], _> = hash.try_into() else {
                        eprintln!(
                            "Invalid SHA512 hash was provided for {name}. I must be 64 bytes."
                        );
                        panic!("{EXIT_MSG}");
                    };
                    Some(Credential::SHA512(hash.into()))
                }
                Err(e) => {
                    eprintln!("Invalid SHA512 hash was provided for {name}: {e}");
                    panic!("{EXIT_MSG}");
                }
            },
            None => {
                warn!("Using plaintext for {name} verification! {recommended} is recommended.");
                Some(Credential::Plain(val))
            }
            _ => {
                warn!("Bad hashing algorithm was provided for {name}! Treating it as plaintext.");
                Some(Credential::Plain(val))
            }
        }
    }

    pub(crate) async fn validate(
        &self,
        provided: String,
        permit: Arc<Semaphore>,
    ) -> Result<bool, ()> {
        let Ok(permit) = acquire_permit(permit).await else {
            warn!("Auth verifier threads are full!");
            return Err(());
        };
        let cred = self.to_owned();
        match cred {
            // Semaphore is checked before this in is_api_ok
            Self::Argon2(hash) => spawn_blocking(move || {
                let _permit = permit;
                Argon2::default()
                    .verify_password(provided.as_bytes(), hash.as_ref())
                    .is_ok()
            })
            .await
            .map_err(|e| warn!("Something went wrong while verfying credential: {e}")),
            Self::SHA512(hash) => Ok(constant_time_eq(
                Sha512::digest(provided.as_bytes()).as_slice(),
                hash.as_ref(),
            )),
            Self::Plain(value) => {
                // If hashing is not enabled, use the plaintext API key for matching
                Ok(constant_time_eq(value.as_bytes(), provided.as_bytes()))
            }
        }
    }
}

// Read API key from header and process it
async fn is_api_ok(
    headers: &HeaderMap,
    api_key: Option<Credential>,
    data: web::Data<AppState>,
) -> JSONResponse {
    let api_header = headers.get(API_HEADER).and_then(|h| h.to_str().ok());
    // If the api_key environment variable exists
    if let Some(key) = api_key {
        // If the header exists
        if let Some(header) = api_header {
            // If the header is correct
            match is_key_valid(header.to_string(), key, data.permits.auth.clone()).await {
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
async fn is_key_valid(
    provided_key: String,
    api_key: Credential,
    permit: Arc<Semaphore>,
) -> Result<(), String> {
    // Check if API Key is hashed using Argon2. More algorithms maybe added later.
    let Ok(authorized) = api_key.validate(provided_key, permit).await else {
        warn!("Error verifying API key.");
        return Err("API key validation temporarily unavailable.".to_string());
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
fn is_cookie_valid(cookie: Cookie, key: &Key) -> bool {
    let mut jar = CookieJar::new();
    jar.add_original(cookie.into_owned());
    if let Some(token) = jar.private(key).get(COOKIE_NAME) {
        let token_parts = token.value().split_once(';');
        if let Some((marker, expiry)) = token_parts {
            let time_now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_secs();
            (marker == COOKIE_VALUE_PASSWORD || marker == COOKIE_VALUE_OIDC)
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
            .expect("Appstate wasn't created yet. THIS SHOULD NEVER OCCUR!!!")
            .clone();
        let api_key = data.config.api_key.clone();
        let no_password = data.config.password.is_none();
        let no_oidc = data.oidc_data.is_none();

        Box::pin(async move {
            let api_result = is_api_ok(req.headers(), api_key, data.clone()).await;

            if api_result.success {
                return Ok(Auth::ValidAPIKey);
            }

            if api_result.error {
                return Ok(Auth::InvalidAPIKey { result: api_result });
            }

            if no_password && no_oidc {
                return Ok(Auth::NoAuthNeeded);
            }

            if let Some(cookie) = req.cookie(COOKIE_NAME)
                && is_cookie_valid(cookie, &data.secret_key)
            {
                return Ok(Auth::ValidSession);
            }

            Ok(Auth::None { result: api_result })
        })
    }
}

// Generate a new token for usage in cookie
pub(crate) fn gen_cookie(remember: bool, oidc: bool, secret_key: &Key) -> Cookie<'static> {
    let token_text = if oidc {
        COOKIE_VALUE_OIDC
    } else {
        COOKIE_VALUE_PASSWORD
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
    let mut val = String::with_capacity(token_text.len() + time.len() + 1);
    val.push_str(token_text);
    val.push(';');
    val.push_str(&time);

    let mut jar = CookieJar::new();
    let mut cookie = Cookie::build(COOKIE_NAME, val)
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);
    if remember {
        cookie = cookie.max_age(Duration::days(7));
    }

    jar.private_mut(secret_key).add(cookie.finish());
    jar.delta()
        .next()
        .expect("Cookie jar delta should always succeed.")
        .to_owned()
}
