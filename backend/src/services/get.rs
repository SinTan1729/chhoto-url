// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    Either, HttpRequest, HttpResponse, Responder, get,
    http::header::{Accept, Header},
    mime::{APPLICATION_JSON, TEXT_PLAIN},
    web::{self, Redirect},
};
use log::{debug, info, warn};
use openidconnect::{
    CsrfToken, Nonce, PkceCodeChallenge, RedirectUrl, Scope, core::CoreAuthenticationFlow,
};
use rusqlite::Connection;

use crate::{
    AppState,
    auth::{self, Auth},
    database,
    services::{
        types::{
            BackendConfig,
            ChhotoError::{ClientError, ServerError},
            GetReqParams, OidcCallbackParams, WhoamiRes,
        },
        utils::{self, derive_origin_info, oidc_redirect_once},
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
            .content_type(TEXT_PLAIN)
            .body("Unauthorized"),
        Auth::InvalidAPIKey { result } => HttpResponse::Unauthorized()
            .content_type(TEXT_PLAIN)
            .body(result.reason),
        _ => match utils::getall_helper(&data.reader, params.into_inner()) {
            Ok(s) => HttpResponse::Ok().content_type(APPLICATION_JSON).body(s),
            Err(ServerError) => HttpResponse::InternalServerError()
                .content_type(TEXT_PLAIN)
                .body("Something went wrong while loading the links.".to_owned()),
            Err(ClientError { reason }) => HttpResponse::BadRequest()
                .content_type(TEXT_PLAIN)
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
            .content_type(TEXT_PLAIN)
            .body(url.clone())
    } else {
        HttpResponse::Ok().content_type(TEXT_PLAIN).body("unset")
    }
}

// Get the version number
// This is deprecated, and might be removed in the future.
// Use /api/getconfig instead
#[get("/api/version")]
pub(crate) async fn version() -> HttpResponse {
    let version = utils::get_version();
    let mut out = String::with_capacity(12 + version.len());
    out.push_str("Chhoto URL v");
    out.push_str(&version);
    HttpResponse::Ok().content_type(TEXT_PLAIN).body(out)
}

// Get the user's current role
#[get("/api/whoami")]
pub(crate) async fn whoami(
    data: web::Data<AppState>,
    req: HttpRequest,
    auth: Auth,
) -> HttpResponse {
    let config = &data.config;
    if Accept::parse(&req)
        .is_ok_and(|accept| accept.iter().any(|item| item.item == APPLICATION_JSON))
    {
        let res = WhoamiRes {
            admin: matches!(auth, Auth::ValidSession | Auth::ValidAPIKey),
            public_mode_enabled: config.public_mode,
            password_enabled: config.password.is_some(),
            oidc_enabled: data.oidc_data.is_some(),
        };
        HttpResponse::Ok().json(res)
        // Deprecated: Compatibility mode for older versions, might be removed in a later major release
    } else {
        let res = if matches!(auth, Auth::ValidSession | Auth::ValidAPIKey) {
            "admin"
        } else if config.public_mode {
            "public"
        } else {
            "nobody"
        };
        HttpResponse::Ok().content_type(TEXT_PLAIN).body(res)
    }
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
        Auth::ValidSession | Auth::ValidAPIKey | Auth::NoAuthNeeded => ok_response(),
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
#[get("/oidc/login")]
pub(crate) async fn oidc_redirect(
    auth: Auth,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> HttpResponse {
    let Some(oidc) = data.oidc_data.as_ref() else {
        return HttpResponse::Forbidden()
            .content_type(TEXT_PLAIN)
            .body("OIDC is disabled.");
    };
    let config = &data.config;
    if config.disable_frontend {
        return HttpResponse::Forbidden()
            .content_type(TEXT_PLAIN)
            .body("Frontend is disabled.");
    }

    let (origin, callback) = derive_origin_info(&req.connection_info());
    let Some(url) = RedirectUrl::new(callback)
        .ok()
        .filter(|_| config.oidc_allowed_origins.contains(&origin))
    else {
        warn!("Bad OIDC origin URL: {origin}");
        return HttpResponse::BadRequest()
            .content_type(TEXT_PLAIN)
            .body("Bad OIDC origin URL.");
    };

    let oidc = &mut *oidc.lock().await;
    let client = oidc.oidc_client.clone().set_redirect_uri(url);
    if matches!(auth, Auth::ValidSession) {
        return oidc_redirect_once(&origin).finish();
    }

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut client = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );

    if let Some((k, _)) = config.oidc_custom_claim.as_ref() {
        client = client
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new(k.clone()));
    }

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client.set_pkce_challenge(pkce_challenge).url();
    let state = csrf_token.into_secret();

    let secrets = &mut oidc.states;
    secrets.insert(state, (pkce_verifier, nonce));

    debug!("Attempting OIDC login.");
    oidc_redirect_once(auth_url.as_str()).finish()
}

//OIDC callback
#[get("/oidc/callback")]
pub(crate) async fn oidc_callback(
    auth: Auth,
    data: web::Data<AppState>,
    params: web::Query<OidcCallbackParams>,
    req: HttpRequest,
) -> HttpResponse {
    let config = &data.config;
    let (mut origin, callback) = derive_origin_info(&req.connection_info());
    if config.oidc_issuer_url.is_none() {
        return HttpResponse::Forbidden()
            .content_type(TEXT_PLAIN)
            .body("OIDC is disabled.");
    };
    if config.disable_frontend {
        HttpResponse::Forbidden()
            .content_type(TEXT_PLAIN)
            .body("Frontend is disabled.");
    }
    if matches!(auth, Auth::ValidSession) {
        info!("Successful OIDC login. (Already had valid session.)");
        return oidc_redirect_once(&origin).finish();
    }

    let Some(oidc_data) = &data.oidc_data else {
        info!("Failed OIDC login.");
        return HttpResponse::InternalServerError()
            .content_type(TEXT_PLAIN)
            .body("OIDC login failed.");
    };

    let Some(url) = RedirectUrl::new(callback)
        .ok()
        .filter(|_| config.oidc_allowed_origins.contains(&origin))
    else {
        warn!("Bad OIDC origin URL: {origin}");
        return HttpResponse::BadRequest()
            .content_type(TEXT_PLAIN)
            .body("Bad OIDC origin URL.");
    };

    let oidc_data = &mut *oidc_data.lock().await;
    if let Err(e) = utils::oidc_verify(
        oidc_data.oidc_client.clone().set_redirect_uri(url),
        params.into_inner(),
        oidc_data,
        &config.oidc_custom_claim,
    )
    .await
    {
        warn!("Failed OIDC login: {e}");
        return HttpResponse::Forbidden().body(if e.as_str() == "Custom claim mismatch." {
            "You don't have enough permissions."
        } else {
            "OIDC login failed."
        });
    }

    let cookie = auth::gen_cookie(true, true, &data.secret_key);
    info!("Successful OIDC login.");

    if config.custom_landing_directory.is_some() {
        origin.push_str("/admin/manage");
    }
    oidc_redirect_once(&origin)
        .cookie(cookie.to_owned())
        .finish()
}

// Healthcheck endpoint
#[get("/healthz")]
pub(crate) async fn health_handler(db: web::Data<Connection>) -> impl Responder {
    if database::is_database_healthy(&db) {
        HttpResponse::Ok().content_type(TEXT_PLAIN).body("healthy")
    } else {
        HttpResponse::InternalServerError()
            .content_type(TEXT_PLAIN)
            .body("unhealthy")
    }
}
