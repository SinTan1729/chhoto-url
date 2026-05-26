// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_session::Session;
use actix_web::{
    Either, HttpRequest, HttpResponse, Responder, get,
    http::StatusCode,
    web::{self, Redirect},
};

use crate::{
    AppState,
    auth::{self, is_session_valid},
    config::SlugStyle,
    database,
    services::types::{
        BackendConfig,
        ChhotoError::{ClientError, ServerError},
        GetReqParams,
    },
    utils,
};

// Return all active links
#[get("/api/all")]
pub async fn getall(
    data: web::Data<AppState>,
    session: Session,
    params: web::Query<GetReqParams>,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    // Call is_api_ok() function, pass HttpRequest
    let result = auth::is_api_ok(http, config);
    // If success, return all links
    if result.error {
        HttpResponse::Unauthorized().json(result)
    } else if result.success || auth::is_session_valid(session, config) {
        match utils::getall_helper(&data.db, params.into_inner()) {
            Ok(s) => HttpResponse::Ok().body(s),
            Err(ServerError) => HttpResponse::InternalServerError()
                .body("Something went wrong while loading the links.".to_string()),
            Err(ClientError { reason }) => HttpResponse::BadRequest().body(reason),
        }
    } else {
        HttpResponse::Unauthorized().body("Not logged in!")
    }
}

// Get the site URL
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/siteurl")]
pub async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
    if let Some(url) = &data.config.site_url {
        HttpResponse::Ok().body(url.clone())
    } else {
        HttpResponse::Ok().body("unset")
    }
}

// Get the version number
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/version")]
pub async fn version() -> HttpResponse {
    HttpResponse::Ok().body(format!("Chhoto URL v{}", utils::get_version()))
}

// Get the user's current role
#[get("/api/whoami")]
pub async fn whoami(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    let acting_user = if result.success || is_session_valid(session, config) {
        "admin"
    } else if config.public_mode {
        "public"
    } else {
        "nobody"
    };
    HttpResponse::Ok().body(acting_user)
}

// Get some useful backend config
#[get("/api/getconfig")]
pub async fn getconfig(
    data: web::Data<AppState>,
    session: Session,
    http: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let result = auth::is_api_ok(http, config);
    if result.success || is_session_valid(session, config) || data.config.public_mode {
        let backend_config = BackendConfig {
            version: utils::get_version(),
            allow_capital_letters: config.allow_capital_letters,
            public_mode: config.public_mode,
            public_mode_expiry_delay: config.public_mode_expiry_delay.unwrap_or_default(),
            site_url: config.site_url.clone(),
            slug_style: (match config.slug_style {
                SlugStyle::Uid => "UID",
                SlugStyle::Pair => "Pair",
            })
            .to_string(),
            slug_length: config.slug_length,
            try_longer_slug: config.try_longer_slug,
            frontend_page_size: config.frontend_page_size,
        };
        HttpResponse::Ok().json(backend_config)
    } else {
        HttpResponse::Unauthorized().json(result)
    }
}

// Handle a given shortlink
#[get("/{shortlink}")]
pub async fn link_handler(
    shortlink: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let shortlink_str = shortlink.as_str();
    if let Ok(longlink) = database::find_and_add_hit(shortlink_str, &data.db) {
        if data.config.use_temp_redirect {
            Either::Left(Redirect::to(longlink))
        } else {
            // Defaults to permanent redirection
            Either::Left(Redirect::to(longlink).permanent())
        }
    } else {
        Either::Right(
            NamedFile::open_async("./resources/static/404.html")
                .await
                .customize()
                .with_status(StatusCode::NOT_FOUND),
        )
    }
}
