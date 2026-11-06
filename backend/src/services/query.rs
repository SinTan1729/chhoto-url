// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse,
    web::{self, ServiceConfig, post, query},
};
use log::{debug, error};
use tokio::task::spawn_blocking;

use crate::{
    AppState,
    auth::Auth,
    database,
    services::types::{
        ChhotoError::{ClientError, ServerError},
        JSONResponse, LinkInfo,
    },
};

pub(super) fn query_routes_v1(cfg: &mut ServiceConfig) {
    cfg.route("/expand", query().to(expand))
        .route("/expand", post().to(expand)); // For backwards compatibility, deprecated
}
// Get information about a single shortlink
async fn expand(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    debug!("Expanding link {}", req);
    match auth {
        Auth::ValidAPIKey => match spawn_blocking(move || {
            let db = data.readers.get().map_err(|_| {
                error!("{}", crate::DB_POOL_ERROR);
                ServerError
            })?;
            database::find_url(&req, &db)
        })
        .await
        .map_err(|_| ServerError)
        .flatten()
        {
            Ok(chunks) => {
                let body = LinkInfo {
                    success: true,
                    error: false,
                    longurl: chunks.longlink,
                    hits: chunks.hits,
                    enabled: chunks.enabled,
                    expiry_time: chunks.expiry_time,
                    notes: chunks.notes,
                };
                HttpResponse::Ok().json(body)
            }
            Err(ServerError) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason: "Something went wrong when expanding the link".to_owned(),
                };
                HttpResponse::InternalServerError().json(body)
            }
            Err(ClientError { reason }) => {
                let body = JSONResponse {
                    success: false,
                    error: true,
                    reason,
                };
                HttpResponse::BadRequest().json(body)
            }
        },
        Auth::ValidSession | Auth::NoAuthNeeded => {
            HttpResponse::Unauthorized().json(JSONResponse {
                success: false,
                error: true,
                reason: "This route needs API auth.".to_owned(),
            })
        }
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            HttpResponse::Unauthorized().json(result)
        }
    }
}
