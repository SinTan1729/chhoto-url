// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use log::{error, info, warn};
use std::{
    collections::HashSet,
    env::{VarError, var},
    fmt::Display,
    fs,
    path::Path,
    sync::Arc,
};
use url::Url;

use crate::{auth, utils};

#[derive(Clone)]
pub(crate) enum SlugStyle {
    Pair,
    Uid,
}
impl Display for SlugStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pair => "Pair",
                Self::Uid => "UID",
            }
        )
    }
}

fn remove_surrounding_quotes(s: &str) -> String {
    match s.as_bytes().first() {
        Some(b'\'') | Some(b'"') | Some(b'`') if s.len() >= 2 => {
            let q = s.as_bytes()[0];
            if s.as_bytes().last() == Some(&q) {
                return s[1..s.len() - 1].to_string();
            }
        }
        _ => {}
    }
    s.to_string()
}
fn remove_trailing_slash(s: &str) -> String {
    if s.as_bytes().last() == Some(&b'/') {
        return (s[0..s.len() - 1]).to_string();
    }
    s.to_string()
}

// This is needed to support old variable names.
// Might be deprecated at a later point.
fn read_config_wrapper(new_name: &str, old_name: &str) -> Result<String, VarError> {
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

// Get db location, and move from old location if needed
fn get_db_location() -> String {
    if let Some(db_url) = read_config_wrapper("CHHOTO_DB_URL", "db_url")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
    {
        info!("Custom database location was provided.");
        return db_url;
    }

    let (legacy_location, new_location) = ("/urls.sqlite", "/data/urls.sqlite");
    if let Some(parent) = Path::new(new_location).parent() {
        fs::create_dir_all(parent).expect("Unable to create database directory!");
    }
    let legacy_exists = fs::exists(legacy_location).unwrap_or(false);
    let new_exists = fs::exists(new_location).unwrap_or(false);
    let bak_location = format!("{legacy_location}.bak");

    match (legacy_exists, new_exists) {
        (true, false) => {
            let tmp_location = format!("{new_location}.tmp");
            fs::copy(legacy_location, &tmp_location)
                .expect("Unable to copy database to new location!");
            fs::rename(&tmp_location, new_location).expect("Unable to rename the new database.");
            info!("Migrated database from {legacy_location} to {new_location}.");
            if let Err(e) = fs::rename(legacy_location, &bak_location) {
                warn!("Unable to rename the legacy database: {e}");
            } else {
                info!("Kept a backup of the legacy database at {bak_location}.");
            }
        }
        (true, true) => {
            warn!(
                "Found databases at both {legacy_location} and {new_location}. Using {new_location}."
            );
            warn!(
                "Verify that the legacy database is no longer needed, then remove {legacy_location}."
            );
        }
        (false, false) => {
            info!("No existing database was found. Will create a new one.");
        }
        (false, true) => {
            if fs::exists(&bak_location).unwrap_or(false) {
                warn!(
                    "A leftover backup file exists at {bak_location}. You might want to delete it."
                );
            }
            info!("Existing database found at {new_location}.");
        }
    }
    new_location.to_owned()
}

#[derive(Clone)]
pub(crate) enum Credential {
    Plain(String),
    Argon2(Arc<argon2::PasswordHash>),
}
impl Credential {
    fn parse(val: String, hash_algorithm: &Option<String>) -> Option<Self> {
        if hash_algorithm.as_deref() == Some("Argon2") {
            match argon2::PasswordHash::new(&val) {
                Ok(hash) => Some(Credential::Argon2(Arc::new(hash))),
                Err(e) => {
                    warn!("Invalid Argon2 hash was provided: {e}");
                    None
                }
            }
        } else {
            Some(Credential::Plain(val))
        }
    }
}

// Struct for storing config read form env vars that might be accessed more than once
#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) listen_address: String,
    pub(crate) port: u16,
    pub(crate) db_location: String,
    pub(crate) cache_control_header: Option<String>,
    pub(crate) disable_frontend: bool,
    pub(crate) custom_site_title: Option<String>,
    pub(crate) site_url: Option<String>,
    pub(crate) oidc_issuer_url: Option<String>,
    pub(crate) oidc_client_id: Option<String>,
    pub(crate) oidc_client_secret: Option<String>,
    pub(crate) oidc_custom_claim: Option<(String, String)>,
    pub(crate) oidc_allowed_origins: HashSet<String>,
    pub(crate) public_mode: bool,
    pub(crate) public_mode_expiry_delay: Option<i64>,
    pub(crate) use_temp_redirect: bool,
    pub(crate) allowed_protocols: HashSet<String>,
    pub(crate) password: Option<Credential>,
    pub(crate) api_key: Option<Credential>,
    pub(crate) slug_style: SlugStyle,
    pub(crate) slug_length: usize,
    pub(crate) try_longer_slug: bool,
    pub(crate) allow_capital_letters: bool,
    pub(crate) custom_landing_directory: Option<String>,
    pub(crate) use_wal_mode: bool,
    pub(crate) ensure_acid: bool,
    pub(crate) disable_backups: bool,
    pub(crate) frontend_page_size: u16,
}

pub(crate) fn read() -> Config {
    let db_location = get_db_location();
    info!("Database Location is set to: {db_location}");

    // Get the address environment variable
    let listen_address = read_config_wrapper("CHHOTO_LISTEN_ADDRESS", "listen_address")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or(String::from("0.0.0.0"));
    info!("Listening address is set to: {listen_address}");

    // Get the port environment variable
    let port = read_config_wrapper("CHHOTO_LISTEN_PORT", "port")
        .unwrap_or(String::from("4567"))
        .parse::<u16>()
        .expect("Supplied port is not an integer");
    info!("Listening port is set to: {port}");

    let cache_control_header =
        read_config_wrapper("CHHOTO_CACHE_CONTROL_HEADER", "cache_control_header")
            .ok()
            .inspect(|h| info!("Using \"{h}\" as Cache-Control header."))
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());

    let disable_frontend = read_config_wrapper("CHHOTO_DISABLE_FRONTEND", "disable_frontend")
        .is_ok_and(|s| s.trim() == "True");
    if disable_frontend {
        info!("Frontend is disabled.")
    };

    let hash_algorithm = read_config_wrapper("CHHOTO_HASH_ALGORITHM", "hash_algorithm")
        .ok()
        .map(|h| h.trim().to_owned());
    // If an API key is set, check the security
    let api_key = read_config_wrapper("CHHOTO_API_KEY", "api_key")
        .ok()
        .inspect(|key| {
            if hash_algorithm.is_none()
                && base64_url::decode(key.as_bytes())
                    .ok()
                    .filter(|b| b.len() >= 64)
                    .is_none()
            {
                warn!(
                    "Provided API key is insecure! \
                    Generated secure key which you may use: {}",
                    auth::gen_key()
                );
            } else {
                info!("Secure API key was provided.");
            }
        })
        .and_then(|s| Credential::parse(s, &hash_algorithm));

    let public_mode =
        read_config_wrapper("CHHOTO_PUBLIC_MODE", "public_mode") == Ok(String::from("Enable"));
    let public_mode_expiry_delay = read_config_wrapper(
        "CHHOTO_PUBLIC_MODE_EXPIRY_DELAY",
        "public_mode_expiry_delay",
    )
    .ok()
    .and_then(|s| s.parse::<i64>().ok())
    .filter(|&s| s > 0);
    if public_mode {
        if let Some(delay) = public_mode_expiry_delay {
            info!("Enabling public mode with an enforced expiry delay of {delay} seconds.");
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

    let mut allowed_protocols: HashSet<String> = ["http", "https", "ftp", "magnet"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Ok(protocols) = var("CHHOTO_EXTRA_PROTOCOLS") {
        for protocol in protocols.split(&[',', ' ']).filter(|p| !p.is_empty()) {
            if protocol.chars().all(|c| c.is_alphanumeric()) {
                allowed_protocols.insert(protocol.to_owned());
            } else {
                warn!("Skipping malformed protocol: {protocol}.")
            }
        }
    }
    info!("Allowed protocols: {:?}", allowed_protocols);

    let password = read_config_wrapper("CHHOTO_PASSWORD", "password")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(
            || {
                warn!("No password was provided.");
                None
            },
            |s| Credential::parse(s, &hash_algorithm),
        );

    if password.is_some() || api_key.is_some() {
        if hash_algorithm.as_deref() == Some("Argon2") {
            info!("Using Argon2 hashes for password and API key verification.");
        } else {
            warn!("Using plaintext for password and API key verification. Argon2 is advised.");
        }
    }
    // If the site_url env variable exists
    let site_url = if let Some(provided_url) = read_config_wrapper("CHHOTO_SITE_URL", "site_url")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
    {
        let url = remove_surrounding_quotes(&provided_url);
        let url = remove_trailing_slash(&url);

        match Url::parse(&url) {
            Ok(u) => {
                if ["http", "https"].contains(&u.scheme()) {
                    Some(url)
                } else {
                    error!(
                        "Bad scheme {} supplied for provided site url {url}. Ignoring the config.",
                        u.scheme(),
                    );
                    None
                }
            }
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                warn!("No scheme supplied for site url {url}, defaulting to https.");
                let u = format!("https://{url}");
                if Url::parse(&u).is_ok() {
                    Some(u)
                } else {
                    error!("Bad URL even after adding HTTPS. Ignoring the config.");
                    None
                }
            }
            Err(e) => {
                error!("Error while processing the provided site url {url}: {e}");
                None
            }
        }
    } else {
        // Site URL is not configured
        warn!("The CHHOTO_SITE_URL environment variable is not configured.");
        None
    };
    if let Some(u) = &site_url {
        info!("Configured Site URL: {}", u);
    }

    let mut oidc_allowed_origins = HashSet::new();
    let oidc_issuer_url = var("CHHOTO_OIDC_ISSUER_URL")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .map(|s| remove_trailing_slash(&s))
        .filter(|s| !s.is_empty())
        .filter(|_| {
            if site_url.is_none() {
                warn!("Site URL must be non empty for OIDC to work.");
            }
            site_url.is_some()
        })
        .filter(|u| match Url::parse(u) {
            Ok(u) => {
                if u.scheme() == "https" {
                    true
                } else {
                    warn!("OIDC Issuer URL must be HTTPS.");
                    false
                }
            }
            Err(_) => {
                warn!("Invalid OIDC Issuer URL was provided.");
                false
            }
        })
        .inspect(|u| {
            info!("Enabling OIDC with issuer url: {u}");
            oidc_allowed_origins.insert(
                site_url
                    .clone()
                    .expect("Site URL must be non empty at this point."),
            );
        });
    let oidc_client_id = var("CHHOTO_OIDC_CLIENT_ID")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .map(|s| remove_surrounding_quotes(&s))
        .filter(|s| !s.is_empty() && oidc_issuer_url.is_some());
    let oidc_client_secret = var("CHHOTO_OIDC_CLIENT_SECRET")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .filter(|s| !s.is_empty() && oidc_issuer_url.is_some());
    let oidc_custom_claim = var("CHHOTO_OIDC_CUSTOM_CLAIM")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .filter(|s| !s.is_empty() && oidc_issuer_url.is_some())
        .and_then(|s| {
            s.split_once("=")
                .filter(|(f, l)| !f.is_empty() && !l.is_empty())
                .map(|(f, l)| (f.to_string(), l.to_string()))
        })
        .inspect(|(f, l)| info!("OIDC custom claim was configured as: {f}={l}"));
    if let Some(urls) = var("CHHOTO_OIDC_EXTRA_ORIGINS")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .map(|s| remove_trailing_slash(&s))
        .filter(|s| !s.is_empty() && oidc_issuer_url.is_some())
    {
        for url in urls.split(&[',', ' ']).filter(|p| !p.is_empty()) {
            if Url::parse(url).is_ok_and(|u| ["http", "https"].contains(&u.scheme())) {
                oidc_allowed_origins.insert(url.to_owned());
            } else {
                warn!("Skipping malformed protocol: {url}.")
            }
        }
    };
    if oidc_issuer_url.is_some() {
        info!("OIDC allowed origins: {:?}", oidc_allowed_origins);
    }

    let custom_site_title = var("CHHOTO_CUSTOM_SITE_TITLE")
        .ok()
        .map(|s| remove_surrounding_quotes(s.trim()))
        .filter(|s| !s.is_empty())
        .inspect(|t| info!(r#"Found custom site title config: "{t}""#,));

    let slug_length = read_config_wrapper("CHHOTO_SLUG_LENGTH", "slug_length")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&s| s >= 4)
        .unwrap_or(8);
    let try_longer_slug = read_config_wrapper("CHHOTO_TRY_LONGER_SLUG", "try_longer_slug")
        .is_ok_and(|s| s.trim() == "True");
    let slug_style =
        match read_config_wrapper("CHHOTO_SLUG_STYLE", "slug_style").map(|s| s.trim().to_owned()) {
            Ok(style) if style == "UID" => {
                info!("Using UID slugs with length {slug_length}.");
                if try_longer_slug {
                    info!("Will retry with a longer slug upon collision.");
                }
                SlugStyle::Uid
            }
            _ => {
                info!("Using adjective-noun pair slugs.");
                SlugStyle::Pair
            }
        };

    let allow_capital_letters =
        read_config_wrapper("CHHOTO_ALLOW_CAPITAL_LETTERS", "allow_capital_letters")
            .is_ok_and(|s| s.trim() == "True");
    if allow_capital_letters {
        info!("Capital letters will be allowed in links.");
    } else {
        info!("Capital letters won't be allowed in links.");
    }

    let use_wal_mode = read_config_wrapper("CHHOTO_SQLITE_USE_WAL_MODE", "use_wal_mode")
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
    let disable_backups = var("CHHOTO_DISABLE_BACKUPS").is_ok_and(|s| s.trim() == "True");
    if disable_backups {
        warn!("Disabling backups.");
    }

    let custom_landing_directory = read_config_wrapper(
        "CHHOTO_CUSTOM_LANDING_DIRECTORY",
        "custom_landing_directory",
    )
    .ok()
    .map(|s| s.trim().to_owned())
    .filter(|s| !s.is_empty())
    .filter(|s| utils::is_custom_landing_directory_valid(s))
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
        custom_site_title,
        site_url,
        oidc_issuer_url,
        oidc_client_id,
        oidc_client_secret,
        oidc_custom_claim,
        oidc_allowed_origins,
        public_mode,
        public_mode_expiry_delay,
        use_temp_redirect,
        allowed_protocols,
        password,
        api_key,
        slug_style,
        slug_length,
        try_longer_slug,
        allow_capital_letters,
        custom_landing_directory,
        use_wal_mode,
        ensure_acid,
        disable_backups,
        frontend_page_size,
    }
}
