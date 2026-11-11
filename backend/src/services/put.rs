// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    HttpResponse,
    web::{self, ServiceConfig, put},
};
use log::error;
use tokio::task::spawn_blocking;

use crate::{
    AppState,
    auth::Auth,
    background::{HitUpdate, acquire_permit},
    services::{
        types::{
            ChhotoError::{ClientError, ServerError},
            JSONResponse,
        },
        utils,
    },
};

pub(super) fn put_routes_v1(cfg: &mut ServiceConfig) {
    cfg.route("/edit", put().to(edit_link));
}

// Edit a shortlink
async fn edit_link(req: String, auth: Auth, data: web::Data<AppState>) -> HttpResponse {
    match auth {
        Auth::ValidAPIKey | Auth::ValidSession | Auth::NoAuthNeeded => {
            let data_clone = data.clone();
            let server_error = HttpResponse::InternalServerError().json(JSONResponse {
                success: false,
                error: true,
                reason: "Something went wrong when editing the link.".to_owned(),
            });
            let Ok(permit) = acquire_permit(data.permits.db.clone()).await else {
                return server_error;
            };
            match spawn_blocking(move || {
                let _permit = permit;
                utils::edit_link_helper(&req, &data_clone.writer, &data_clone.config)
            })
            .await
            .map_err(|_| ServerError)
            .flatten()
            {
                Ok(result) => {
                    let body = JSONResponse {
                        success: true,
                        error: false,
                        reason: String::from("Edit was successful."),
                    };
                    if let Some(shortlink) = result
                        && let Err(err) = data.hits_tx.send(HitUpdate::Reset(shortlink)).await
                    {
                        error!("Failed to enqueue hit update after edit: {err}");
                    }
                    HttpResponse::Created().json(body)
                }
                Err(ServerError) => server_error,
                Err(ClientError { reason }) => {
                    let body = JSONResponse {
                        success: false,
                        error: true,
                        reason,
                    };
                    HttpResponse::BadRequest().json(body)
                }
            }
        }
        Auth::None { result } | Auth::InvalidAPIKey { result } => {
            HttpResponse::Unauthorized().json(result)
        }
    }
}
