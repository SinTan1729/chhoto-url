// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_files::NamedFile;
use actix_web::{
    Either, HttpResponse, Responder, get,
    http::StatusCode,
    web::{self, Redirect},
};

use crate::{
    AppState,
    auth::Auth,
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
pub(crate) async fn getall(
    auth: Auth,
    data: web::Data<AppState>,
    params: web::Query<GetReqParams>,
) -> HttpResponse {
    match auth {
        Auth::None { result: _ } => HttpResponse::Unauthorized().body("Unauthorized"),
        Auth::InvalidAPIKey { result } => HttpResponse::Unauthorized().body(result.reason),
        _ => match utils::getall_helper(&data.db, params.into_inner()) {
            Ok(s) => HttpResponse::Ok().body(s),
            Err(ServerError) => HttpResponse::InternalServerError()
                .body("Something went wrong while loading the links.".to_string()),
            Err(ClientError { reason }) => HttpResponse::BadRequest().body(reason),
        },
    }
}

// Get the site URL
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/siteurl")]
pub(crate) async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
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
pub(crate) async fn version() -> HttpResponse {
    HttpResponse::Ok().body(format!("Chhoto URL v{}", utils::get_version()))
}

// Get the user's current role
#[get("/api/whoami")]
pub(crate) async fn whoami(data: web::Data<AppState>, auth: Auth) -> HttpResponse {
    let config = &data.config;
    let acting_user = match auth {
        Auth::ValidAPIKey | Auth::ValidSession => "admin",
        _ => {
            if config.public_mode {
                "public"
            } else {
                "nobody"
            }
        }
    };
    HttpResponse::Ok().body(acting_user)
}

// Get some useful backend config
#[get("/api/getconfig")]
pub(crate) async fn getconfig(auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    let config = &data.config;
    let ok_response = || {
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
    };
    match auth {
        Auth::ValidSession | Auth::ValidAPIKey => ok_response(),
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            if data.config.public_mode {
                ok_response()
            } else {
                HttpResponse::Unauthorized().json(result)
            }
        }
    }
}

// Handle a given shortlink
#[get("/{shortlink}")]
pub(crate) async fn link_handler(
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
