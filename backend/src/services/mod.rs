// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

mod delete;
mod get;
mod post;
mod put;
mod query;
pub(crate) mod routes;
pub(crate) mod types;
pub(crate) mod utils;

pub(crate) use self::routes::{
    api_routes_v1, error_route, health_route, link_handler, oidc_routes,
};
