// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    Either, HttpResponse, Responder, get,
    web::{self, Redirect},
};
use log::{debug, info, warn};
use openidconnect::{CsrfToken, Nonce, PkceCodeChallenge, Scope, core::CoreAuthenticationFlow};
use rusqlite::Connection;

use crate::{
    AppState,
    auth::Auth,
    database,
    services::{
        types::{
            BackendConfig,
            ChhotoError::{ClientError, ServerError},
            GetReqParams, OidcParams,
        },
        utils,
    },
};

// Return all active links
#[get("/api/all")]
pub(crate) async fn getall(
    auth: Auth,
    data: web::Data<AppState>,
    params: web::Query<GetReqParams>,
) -> HttpResponse {
    match auth {
        Auth::None { result: _ } => HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized"),
        Auth::InvalidAPIKey { result } => HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body(result.reason),
        _ => match utils::getall_helper(&data.reader, params.into_inner()) {
            Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
            Err(ServerError) => HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body("Something went wrong while loading the links.".to_owned()),
            Err(ClientError { reason }) => HttpResponse::BadRequest()
                .content_type("text/plain")
                .body(reason),
        },
    }
}

// Get the site URL
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/siteurl")]
pub(crate) async fn siteurl(data: web::Data<AppState>) -> HttpResponse {
    if let Some(url) = &data.config.site_url {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(url.clone())
    } else {
        HttpResponse::Ok().content_type("text/plain").body("unset")
    }
}

// Get the version number
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/version")]
pub(crate) async fn version() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Chhoto URL v{}", utils::get_version()))
}

// Get the user's current role
#[get("/api/whoami")]
pub(crate) async fn whoami(data: web::Data<AppState>, auth: Auth) -> HttpResponse {
    let config = &data.config;
    let acting_user = match auth {
        Auth::ValidAPIKey | Auth::ValidSession => "admin",
        Auth::NoPass => {
            if config.public_mode {
                "public-nopass"
            } else {
                "nobody-nopass"
            }
        }
        _ => {
            if config.public_mode {
                "public"
            } else {
                "nobody"
            }
        }
    };
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(acting_user)
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
            allowed_protocols: config.allowed_protocols.clone(),
            site_url: config.site_url.clone(),
            slug_style: config.slug_style.to_string(),
            slug_length: config.slug_length,
            try_longer_slug: config.try_longer_slug,
            frontend_page_size: config.frontend_page_size,
        };
        HttpResponse::Ok().json(backend_config)
    };
    match auth {
        Auth::ValidSession | Auth::ValidAPIKey | Auth::NoPass => ok_response(),
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
    if let Ok(longlink) =
        database::find_and_add_hit(shortlink_str, &data.reader, &data.hits_tx).await
    {
        if data.config.use_temp_redirect {
            Either::Left(Redirect::to(longlink))
        } else {
            // Defaults to permanent redirection
            Either::Left(Redirect::to(longlink).permanent())
        }
    } else {
        Either::Right(utils::error404(data).await)
    }
}

// OIDC redirect
#[get("/oidc/redirect")]
pub(crate) async fn oidc_redirect(auth: Auth, data: web::Data<AppState>) -> impl Responder {
    let Some(oidc) = data.oidc_data.clone() else {
        return Either::Left(HttpResponse::BadRequest().body("OIDC is disabled."));
    };
    let Some(url) = data.config.site_url.clone() else {
        return Either::Left(HttpResponse::BadRequest().body("OIDC is disabled."));
    };
    if matches!(auth, Auth::ValidSession) {
        return Either::Right(Redirect::to(url));
    }

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut client = oidc
        .oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("read".to_string()));

    if let Some((k, _)) = data.config.oidc_custom_claim.as_ref() {
        client = client
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new(k.clone()));
    }

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client.set_pkce_challenge(pkce_challenge).url();
    let state = csrf_token.into_secret();

    let secrets = &mut *oidc.states.lock().await;
    secrets.insert(state, (pkce_verifier, nonce));

    debug!("Attempting OIDC login.");
    Either::Right(Redirect::to(auth_url.to_string()))
}

//OIDC callback
#[get("/oidc/callback")]
pub(crate) async fn oidc_callback(
    auth: Auth,
    data: web::Data<AppState>,
    params: web::Query<OidcParams>,
    session: actix_session::Session,
) -> impl Responder {
    let Some(url) = data.config.site_url.clone() else {
        return Either::Left(HttpResponse::BadRequest().body("OIDC is disabled."));
    };
    if matches!(auth, Auth::ValidSession) {
        info!("Successful OIDC login. (Already had valid session.)");
        return Either::Right(Redirect::to(url));
    }

    let Some(oidc_data) = data.oidc_data.clone() else {
        info!("Failed OIDC login.");
        return Either::Left(HttpResponse::InternalServerError().body("OIDC login failed."));
    };

    if let Err(e) = utils::oidc_verify(
        params.into_inner(),
        oidc_data,
        data.config.oidc_custom_claim.clone(),
    )
    .await
    {
        warn!("OIDC error: {e}");
        info!("Failed OIDC login.");
        return Either::Left(HttpResponse::Forbidden().body("OIDC login failed."));
    }

    if session
        .insert("chhoto-url-auth", crate::auth::gen_token_text(true))
        .is_err()
    {
        info!("Failed OIDC login.");
        return Either::Left(HttpResponse::Forbidden().body("OIDC login failed."));
    };

    info!("Successful OIDC login.");
    Either::Right(Redirect::to(url))
}

// Healthcheck endpoint
#[get("/healthz")]
pub(crate) async fn health_handler(db: web::Data<Connection>) -> impl Responder {
    if database::is_database_healthy(&db) {
        HttpResponse::Ok().message_body("healthy")
    } else {
        HttpResponse::InternalServerError().message_body("unhealthy")
    }
}
