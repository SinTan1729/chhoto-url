// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, fmt::Display, rc::Rc};

use actix_http::{
    Request, StatusCode,
    header::{HeaderValue, InvalidHeaderValue, TryIntoHeaderValue},
};
use actix_service::Service;
use actix_web::{App, Error, body::to_bytes, cookie::Key, dev::ServiceResponse, test, web::Bytes};
use serde::Deserialize;
use tempfile::TempDir;

use crate::{config::Credential, *};

pub(super) const API_HEADER: &str = "X-API-Key";

pub(super) trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

impl TryIntoHeaderValue for Credential {
    type Error = InvalidHeaderValue;
    fn try_into_value(self) -> std::prelude::v1::Result<HeaderValue, Self::Error> {
        match self {
            Credential::Plain(s) => HeaderValue::from_str(&s),
            Credential::Argon2(_) => HeaderValue::from_str("Argon2 cannot be converted to header."),
        }
    }
}
impl TryIntoHeaderValue for &Credential {
    type Error = InvalidHeaderValue;
    fn try_into_value(self) -> std::prelude::v1::Result<HeaderValue, Self::Error> {
        Credential::try_into_value((*self).clone())
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
        custom_site_title: None,
        site_url: Some(String::from("https://mydomain.com")),
        oidc_issuer_url: None,
        oidc_client_id: None,
        oidc_client_secret: None,
        oidc_custom_claim: None,
        oidc_allowed_origins: HashSet::from([String::from("https://mydomain.com")]),
        public_mode: false,
        public_mode_expiry_delay: None,
        use_temp_redirect: false,
        allowed_protocols: HashSet::from(["http", "https", "ftp", "magnet"].map(|s| s.to_string())),
        password: Some(Credential::Plain(String::from("testpass"))),
        api_key: Some(Credential::Plain(base64_url::encode(
            Key::generate().master(),
        ))),
        slug_style: config::SlugStyle::Pair,
        slug_length: 8,
        try_longer_slug: false,
        allow_capital_letters: false,
        custom_landing_directory: None,
        use_wal_mode: true,
        ensure_acid: false,
        disable_backups: true,
        frontend_page_size: 10,
    }
}

pub(super) async fn create_app(
    conf: &config::Config,
    test: &str,
) -> (
    TempDir,
    impl Service<Request, Response = ServiceResponse, Error = Error> + use<>,
) {
    let tempdir = TempDir::new().unwrap();
    let db_file = tempdir.path().join(format!("{test}.sqlite"));
    let writer = Arc::from(Mutex::from(database::open_db(
        db_file.to_str().unwrap(),
        false,
    )));
    database::init_db(
        &mut *writer.lock().await,
        conf.use_wal_mode,
        conf.ensure_acid,
        conf.disable_backups,
    );

    let (hits_tx, hits_rx) = mpsc::channel::<(String, bool)>(1024);
    background::spawn_hits_worker(Arc::clone(&writer), hits_rx);

    (
        tempdir,
        test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    hits_tx,
                    reader: database::open_db(db_file.to_str().unwrap(), false),
                    writer,
                    config: Arc::new(conf.clone()),
                    frontend_dir: String::new(),
                    oidc_data: None,
                    secret_key: Arc::new(Key::generate()),
                }))
                .service(services::siteurl)
                .service(services::version)
                .service(services::getconfig)
                .service(services::add_links)
                .service(services::getall)
                .service(services::link_handler)
                .service(services::edit_link)
                .service(services::delete_link)
                .service(services::whoami)
                .service(services::expand)
                .service(services::oidc_redirect),
        )
        .await,
    )
}

pub(super) async fn add_link<
    T: Service<Request, Response = ServiceResponse, Error = Error>,
    S: Display,
>(
    app: T,
    api_key: &Credential,
    shortlink: S,
    expiry_delay: i64,
    notes: &str,
) -> (StatusCode, URLData) {
    let req = test::TestRequest::post().uri("/api/new")
        .insert_header((API_HEADER, api_key))
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
    api_key: &Credential,
    shortlink: S,
) -> (StatusCode, URLData) {
    let req = test::TestRequest::query()
        .uri("/api/expand")
        .insert_header((API_HEADER, api_key))
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
    api_key: &Credential,
    params: &str,
) -> Rc<[URLData]> {
    let req = test::TestRequest::get()
        .uri(&format!("/api/all?{params}"))
        .insert_header((API_HEADER, api_key))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: Rc<[URLData]> = serde_json::from_str(body.as_str()).unwrap();

    reply_chunks
}

pub(super) async fn edit_link<T: Service<Request, Response = ServiceResponse, Error = Error>>(
    app: T,
    api_key: &Credential,
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
        .insert_header((API_HEADER, api_key))
        .set_payload(format!("{{{payload}}}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    resp.status()
}
