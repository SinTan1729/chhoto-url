// SPDX-FileCopyrightText: 2025 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{info, warn};
use passwords::{analyzer::analyze, scorer::score};
use std::env::var;

use crate::auth;

// Struct for storing config read form env vars that might be accessed more than once
#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub db_location: String,
    pub cache_control_header: Option<String>,
    pub disable_frontend: bool,
    pub site_url: Option<String>,
    pub public_mode: bool,
    pub public_mode_expiry_delay: i64,
    pub use_temp_redirect: bool,
    pub password: Option<String>,
    pub hash_algorithm: Option<String>,
    pub api_key: Option<String>,
    pub slug_style: String,
    pub slug_length: usize,
    pub try_longer_slug: bool,
}

pub fn read() -> Config {
    let db_location = var("db_url")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(String::from("urls.sqlite"));
    info!("DB Location is set to: {db_location}");

    // Get the port environment variable
    let port = var("port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");
    info!("Listening port is set to {port}.");

    let cache_control_header = var("cache_control_header")
        .ok()
        .inspect(|h| info!("Using \"{h}\" as Cache-Control header."))
        .filter(|s| !s.trim().is_empty());

    let disable_frontend = var("disable_frontend").is_ok_and(|s| s.trim() == "True");
    if disable_frontend {
        info!("Frontend is disabled.")
    };

    // If an API key is set, check the security
    let api_key = var("api_key").ok();
    if let Some(key) = &api_key {
        // Determine whether the inputted API key is sufficiently secure
        if score(&analyze(key)) < 90.0 {
            warn!("API key is insecure! Please change it. Current key is: {}. Generated secure key which you may use: {}", key, auth::gen_key());
        } else {
            info!("Secure API key was provided.");
        }
    }

    let public_mode = var("public_mode") == Ok(String::from("Enable"));
    let public_mode_expiry_delay = var("public_mode_expiry_delay")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_default();
    if public_mode {
        if public_mode_expiry_delay > 0 {
            info!("Enabling public mode with an enforced expiry delay of {public_mode_expiry_delay} seconds.");
        } else {
            info!("Enabling public mode with no enforced expiry delay.");
        }
    }

    let use_temp_redirect = var("redirect_method") == Ok(String::from("TEMPORARY"));
    if use_temp_redirect {
        info!("Using Temporary redirection.");
    } else {
        info!("Using Permanent redirection (default).")
    }

    let password = var("password").ok().filter(|s| !s.trim().is_empty());
    if password.is_none() {
        warn!("No password was provided. The API will be accessible to the public.")
    };

    let hash_algorithm = var("hash_algorithm")
        .ok()
        .filter(|h| h == "Argon2")
        .inspect(|h| info!("Will use {h} hashes for password verification."));

    // If the site_url env variable exists
    let site_url = if let Some(provided_url) = var("site_url").ok().filter(|s| !s.trim().is_empty())
    {
        // Get first and last characters of the site_url
        let mut chars = provided_url.chars();
        let first = chars.next();
        let last = chars.next_back();
        let url = chars.as_str();
        // If the site_url is encapsulated by quotes (i.e. invalid)
        if first == Option::from('"') || first == Option::from('\'') && first == last {
            // Set the site_url without the quotes
            warn!("The site_url environment variable is encapsulated by quotes. Automatically adjusting to: {url}");
            Some(url.to_string())
        } else {
            info!("Configured Site URL is: {provided_url}");
            Some(provided_url)
        }
    } else {
        // Site URL is not configured
        warn!(
            "The site_url environment variable is not configured. Using http://localhost by default."
        );
        let protocol = if port == 443 { "https" } else { "http" };
        let port_text = if [80, 443].contains(&port) {
            String::new()
        } else {
            format!(":{}", port)
        };
        // No issues
        info!("Public URI is: {protocol}://localhost{port_text}.");
        None
    };

    let slug_style = var("slug_style").unwrap_or(String::from("Pair"));
    let slug_length = var("slug_length")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&s| s >= 4)
        .unwrap_or(8);

    let try_longer_slug = var("try_longer_slug").is_ok_and(|s| s.trim() == "True");

    if slug_style == "UID" {
        info!("Using UID slugs with length {slug_length}.");
        if try_longer_slug {
            info!("Will retry with a longer slug upon collision.")
        };
    } else {
        info!("Using adjective-noun pair slugs.");
    }

    Config {
        port,
        db_location,
        cache_control_header,
        disable_frontend,
        site_url,
        public_mode,
        public_mode_expiry_delay,
        use_temp_redirect,
        password,
        hash_algorithm,
        api_key,
        slug_style,
        slug_length,
        try_longer_slug,
    }
}
