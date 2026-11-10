// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{
    Resource, Route,
    web::{ServiceConfig, get, head, resource},
};

use super::{delete, get, post, put, query, utils};

pub(crate) fn api_routes_v1(cfg: &mut ServiceConfig) {
    // Post
    cfg.configure(get::get_routes_v1);
    cfg.configure(post::post_routes_v1);
    cfg.configure(query::query_routes_v1);
    cfg.configure(delete::delete_routes_v1);
    cfg.configure(put::put_routes_v1);
}

pub(crate) fn oidc_routes(cfg: &mut ServiceConfig) {
    cfg.configure(get::oidc_routes);
}

pub(crate) fn error_route() -> Route {
    get().to(utils::error404)
}

pub(crate) fn health_route() -> Route {
    get().to(get::health_handler)
}

pub(crate) fn link_handler() -> Resource {
    resource("/{shortlink}")
        .route(get().to(get::link_handler))
        .route(head().to(get::link_handler))
}
