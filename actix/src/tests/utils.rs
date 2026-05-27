// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_http::{Request, StatusCode};
use actix_service::Service;
use actix_web::{App, Error, body::to_bytes, dev::ServiceResponse, test, web::Bytes};
use serde::Deserialize;
use std::{fmt::Display, fs, rc::Rc};

use crate::*;

pub(super) trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

#[derive(Deserialize)]
pub(super) struct URLData {
    #[serde(default, alias = "shorturl")]
    pub(super) shortlink: String,
    #[serde(default, alias = "longurl")]
    pub(super) longlink: String,
    #[serde(default)]
    pub(super) hits: i64,
    #[serde(default)]
    pub(super) expiry_time: i64,
    #[serde(default)]
    pub(super) notes: String,
    #[serde(default)]
    pub(super) reason: String,
}

#[derive(Deserialize)]
pub(super) struct BackendConfig {
    pub(super) version: String,
    pub(super) slug_length: usize,
}

pub(super) fn default_config(test: &str) -> config::Config {
    config::Config {
        listen_address: String::from("0.0.0.0"),
        port: 4567,
        db_location: format!("/tmp/chhoto-url-test-{test}.sqlite"),
        cache_control_header: None,
        disable_frontend: true,
        site_url: Some(String::from("https://mydomain.com")),
        public_mode: false,
        public_mode_expiry_delay: None,
        use_temp_redirect: false,
        password: Some(String::from("testpass")),
        hash_algorithm: config::HashAlgorithm::None,
        api_key: Some(String::from(
            "Z8FNjh2J2v3yfb0xPDIVA58Pj4D0e2jSERVdoqM5pJCbU2w5tmg3PNioD6GUhaQwHHaDLBNZj0EQE8MS4TLKcUyusa05",
        )),
        slug_style: config::SlugStyle::Pair,
        slug_length: 8,
        try_longer_slug: false,
        allow_capital_letters: false,
        custom_landing_directory: None,
        use_wal_mode: true,
        ensure_acid: false,
        frontend_page_size: 10,
    }
}

pub(super) async fn create_app(
    conf: &config::Config,
    test: &str,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> + use<> {
    let _ = fs::create_dir("/tmp/chhoto-url-test");
    test_cleanup(test);
    let db_file = format!("/tmp/chhoto-url-test/{test}.sqlite");
    database::initialize_db(&db_file, conf.use_wal_mode, conf.ensure_acid);

    test::init_service(
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(&db_file),
                config: conf.clone(),
            }))
            .service(services::siteurl)
            .service(services::version)
            .service(services::getconfig)
            .service(services::add_link)
            .service(services::getall)
            .service(services::link_handler)
            .service(services::edit_link)
            .service(services::delete_link)
            .service(services::whoami)
            .service(services::expand),
    )
    .await
}

pub(super) fn test_cleanup(test: &str) {
    for suffix in ["", ".bak1", ".bak2", "-shm", "-wal"] {
        let _ = fs::remove_file(format!("/tmp/chhoto-url-test/{test}.sqlite{suffix}"));
    }
}

pub(super) async fn add_link<
    T: Service<Request, Response = ServiceResponse, Error = Error>,
    S: Display,
>(
    app: T,
    api_key: &str,
    shortlink: S,
    expiry_delay: i64,
    notes: &str,
) -> (StatusCode, URLData) {
    let req = test::TestRequest::post().uri("/api/new")
        .insert_header(("X-API-Key", api_key))
        .set_payload(format!(
            "{{\"shortlink\":\"{shortlink}\",\"longlink\":\"https://example-{shortlink}.com\",\"expiry_delay\":{expiry_delay},\"notes\":\"{notes}\"}}"
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let url: URLData = serde_json::from_str(body.as_str()).unwrap();

    (status, url)
}

pub(super) async fn expand<
    T: Service<Request, Response = ServiceResponse, Error = Error>,
    S: Display,
>(
    app: T,
    api_key: &str,
    shortlink: S,
) -> (StatusCode, URLData) {
    let req = test::TestRequest::post()
        .uri("/api/expand")
        .insert_header(("X-API-Key", api_key))
        .set_payload(shortlink.to_string())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let url: URLData = serde_json::from_str(body.as_str()).unwrap();

    (status, url)
}

pub(super) async fn getall<T: Service<Request, Response = ServiceResponse, Error = Error>>(
    app: T,
    api_key: &str,
    params: &str,
) -> Rc<[URLData]> {
    let req = test::TestRequest::get()
        .uri(&format!("/api/all?{params}"))
        .insert_header(("X-API-Key", api_key))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: Rc<[URLData]> = serde_json::from_str(body.as_str()).unwrap();

    reply_chunks
}

pub(super) async fn edit_link<T: Service<Request, Response = ServiceResponse, Error = Error>>(
    app: T,
    api_key: &str,
    shortlink: &str,
    reset_hits: bool,
    expiry_time: Option<i64>,
    notes: Option<&str>,
) -> StatusCode {
    let mut payload = format!(
        "\"shortlink\":\"{shortlink}\",\"longlink\":\"https://edited-{shortlink}.com\",\"reset_hits\":{reset_hits}"
    );
    if let Some(expiry) = expiry_time {
        payload.push_str(&format!(",\"expiry_time\":{expiry}"));
    }
    if let Some(note) = notes {
        payload.push_str(&format!(",\"notes\":\"{note}\""));
    }
    let req = test::TestRequest::put()
        .uri("/api/edit")
        .insert_header(("X-API-Key", api_key))
        .set_payload(format!("{{{payload}}}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    resp.status()
}
