// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

// Error types
#[derive(Clone)]
pub(crate) enum ChhotoError {
    ServerError,
    ClientError { reason: String },
}

// Enum for optional batching
#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}
impl<T> OneOrMany<T> {
    pub(super) fn normalize(self) -> Vec<T> {
        match self {
            Self::One(item) => Vec::from([item]),
            Self::Many(items) => items,
        }
    }
}

// Define JSON struct for returning success/error data
#[derive(Serialize)]
pub(crate) struct JSONResponse {
    pub(crate) success: bool,
    pub(crate) error: bool,
    pub(crate) reason: String,
}

// Define JSON struct for returning backend config
#[derive(Serialize)]
pub(super) struct BackendConfig {
    pub(super) version: String,
    pub(super) site_url: Option<String>,
    pub(super) allow_capital_letters: bool,
    pub(super) public_mode: bool,
    pub(super) public_mode_expiry_delay: i64,
    pub(super) allowed_protocols: Vec<String>,
    pub(super) slug_style: String,
    pub(super) slug_length: usize,
    pub(super) try_longer_slug: bool,
    pub(super) frontend_page_size: u16,
}

// Needed to return the short URL to make it easier for programs leveraging the API
#[derive(Serialize)]
pub(super) struct CreatedURL {
    pub(super) success: bool,
    pub(super) error: bool,
    pub(super) shorturl: String,
    pub(super) expiry_time: i64,
}

// Response type for add_links
#[derive(Serialize)]
#[serde(untagged)]
pub(super) enum AddLinkResponse {
    Success(CreatedURL),
    Error(JSONResponse),
}

// Struct for returning information about a shortlink in expand
#[derive(Serialize)]
pub(super) struct LinkInfo {
    pub(super) success: bool,
    pub(super) error: bool,
    pub(super) longurl: String,
    pub(super) hits: i64,
    pub(super) expiry_time: i64,
    pub(super) notes: String,
}

// Struct for query params in /api/all
#[derive(Deserialize)]
pub(crate) struct GetReqParams {
    pub(crate) page_after: Option<String>,
    pub(crate) page_no: Option<i64>,
    pub(crate) page_size: Option<i64>,
    pub(crate) filter: Option<String>,
}
