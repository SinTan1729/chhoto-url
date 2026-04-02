// SPDX-FileCopyrightText: 2025 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{info, warn};
use passwords::{analyzer::analyze, scorer::score};
use std::env::{var, VarError};

use crate::auth;

fn read_config_wrapper(new_name: &str, old_name: &str) -> Result<String, VarError> {
    // This is needed to support old variable names.
    // Might be deprecated at a later point.
    var(new_name).or_else(|e| match e {
        VarError::NotPresent => var(old_name).inspect(|_| {
            warn!(
                "Variable {new_name} was not found, falling back to reading variable {old_name}."
            );
            warn!("Please consider updating your configs.");
        }),
        _ => Err(e),
    })
}

// Struct for storing config read form env vars that might be accessed more than once
#[derive(Clone)]
pub struct Config {
    pub listen_address: String,
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
    pub allow_capital_letters: bool,
    pub custom_landing_directory: Option<String>,
    pub use_wal_mode: bool,
    pub ensure_acid: bool,
    pub frontend_page_size: u16,
}

pub fn read() -> Config {
    let db_location = read_config_wrapper("CHHOTO_DB_URL", "db_url")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(String::from("urls.sqlite"));
    info!("DB Location is set to: {db_location}");

    // Get the address environment variable
    let listen_address = read_config_wrapper("CHHOTO_LISTEN_ADDRESS", "listen_address")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(String::from("0.0.0.0"));
    info!("Listening address is set to {listen_address}.");

    // Get the port environment variable
    let port = read_config_wrapper("CHHOTO_LISTEN_PORT", "port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");
    info!("Listening port is set to {port}.");

    let cache_control_header =
        read_config_wrapper("CHHOTO_CACHE_CONTROL_HEADER", "cache_control_header")
            .ok()
            .inspect(|h| info!("Using \"{h}\" as Cache-Control header."))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

    let disable_frontend = read_config_wrapper("CHHOTO_DISABLE_FRONTEND", "disable_frontend")
        .is_ok_and(|s| s.trim() == "True");
    if disable_frontend {
        info!("Frontend is disabled.")
    };

    // If an API key is set, check the security
    let api_key = read_config_wrapper("CHHOTO_API_KEY", "api_key").ok();
    if let Some(key) = &api_key {
        // Determine whether the inputted API key is sufficiently secure
        if score(&analyze(key)) < 90.0 {
            warn!("API key is insecure! Please change it. Current key is: {}. Generated secure key which you may use: {}", key, auth::gen_key());
        } else {
            info!("Secure API key was provided.");
        }
    }

    let public_mode =
        read_config_wrapper("CHHOTO_PUBLIC_MODE", "public_mode") == Ok(String::from("Enable"));
    let public_mode_expiry_delay = read_config_wrapper(
        "CHHOTO_PUBLIC_MODE_EXPIRY_DELAY",
        "public_mode_expiry_delay",
    )
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

    let use_temp_redirect = read_config_wrapper("CHHOTO_REDIRECT_METHOD", "redirect_method")
        == Ok(String::from("TEMPORARY"));
    if use_temp_redirect {
        info!("Using Temporary redirection.");
    } else {
        info!("Using Permanent redirection (default).")
    }

    let password = read_config_wrapper("CHHOTO_PASSWORD", "password")
        .ok()
        .filter(|s| !s.trim().is_empty());
    if password.is_none() {
        warn!("No password was provided. The API will be accessible to the public.")
    };

    let hash_algorithm = read_config_wrapper("CHHOTO_HASH_ALGORITHM", "hash_algorithm")
        .ok()
        .filter(|h| h == "Argon2")
        .inspect(|h| info!("Will use {h} hashes for password verification."));

    // If the site_url env variable exists
    let site_url = if let Some(provided_url) = read_config_wrapper("CHHOTO_SITE_URL", "site_url")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        // Get first and last characters of the site_url
        let mut chars = provided_url.chars();
        let first = chars.next();
        let last = chars.next_back();
        let url = chars.as_str();
        // If the site_url is encapsulated by quotes (i.e. invalid)
        if first == Option::from('"') || first == Option::from('\'') && first == last {
            // Set the site_url without the quotes
            warn!("The CHHOTO_SITE_URL environment variable is encapsulated by quotes. Automatically adjusting to: {url}");
            Some(url.to_string())
        } else {
            info!("Configured Site URL is: {provided_url}");
            Some(provided_url)
        }
    } else {
        // Site URL is not configured
        warn!(
            "The CHHOTO_SITE_URL environment variable is not configured. Using http://localhost by default."
        );
        let protocol = if port == 443 { "https" } else { "http" };
        let port_text = if [80, 443].contains(&port) {
            String::new()
        } else {
            format!(":{port}")
        };
        // No issues
        info!("Public URL is: {protocol}://localhost{port_text}.");
        None
    };

    let slug_style = read_config_wrapper("CHHOTO_SLUG_STYLE", "slug_style")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(String::from("Pair"));
    let slug_length = read_config_wrapper("CHHOTO_SLUG_LENGTH", "slug_length")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&s| s >= 4)
        .unwrap_or(8);

    let try_longer_slug = read_config_wrapper("CHHOTO_TRY_LONGER_SLUG", "try_longer_slug")
        .is_ok_and(|s| s.trim() == "True");

    if slug_style == "UID" {
        info!("Using UID slugs with length {slug_length}.");
        if try_longer_slug {
            info!("Will retry with a longer slug upon collision.");
        }
    } else {
        info!("Using adjective-noun pair slugs.");
    }

    let allow_capital_letters =
        read_config_wrapper("CHHOTO_ALLOW_CAPITAL_LETTERS", "allow_capital_letters")
            .is_ok_and(|s| s.trim() == "True");
    if allow_capital_letters {
        info!("Capital letters will be allowed in links.");
    } else {
        info!("Capital letters won't be allowed in links.");
    }

    let use_wal_mode = read_config_wrapper("CHHOTO_USE_WAL_MODE", "use_wal_mode")
        .is_ok_and(|s| s.trim() == "True");
    if use_wal_mode {
        info!("Using WAL journaling mode for database.");
    } else {
        warn!("Using DELETE journaling mode for database. WAL mode is recommended.");
    }
    let ensure_acid = !read_config_wrapper("CHHOTO_SQLITE_ENSURE_ACID", "ensure_acid")
        .is_ok_and(|s| s.trim() == "False");
    if ensure_acid {
        let synchronous = if use_wal_mode { "FULL" } else { "EXTRA" };
        info!("Ensuring ACID compliance, using synchronous pragma: {synchronous}.");
    } else {
        let synchronous = if use_wal_mode { "NORMAL" } else { "FULL" };
        info!("Not ensuring ACID compliance, using synchronous pragma: {synchronous}.")
    }

    let custom_landing_directory = read_config_wrapper(
        "CHHOTO_CUSTOM_LANDING_DIRECTORY",
        "custom_landing_directory",
    )
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .inspect(|s| {
        info!("Custom landing directory is set to {s}.");
        info!("The dashboard will be available at /admin/manage/");
    });

    let frontend_page_size = var("CHHOTO_FRONTEND_PAGE_SIZE")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .filter(|&s| s >= 1)
        .inspect(|s| info!("Frontend page size is set to {s}."))
        .unwrap_or(10);

    Config {
        listen_address,
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
        allow_capital_letters,
        custom_landing_directory,
        use_wal_mode,
        ensure_acid,
        frontend_page_size,
    }
}
