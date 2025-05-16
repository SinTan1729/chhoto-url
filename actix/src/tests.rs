use actix_http::{Request, StatusCode};
use actix_service::Service;
use actix_web::{body::to_bytes, dev::ServiceResponse, test, web::Bytes, App, Error};
use regex::Regex;
use serde::Deserialize;
use std::{fmt::Display, fs};

use super::*;

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

#[derive(Deserialize)]
struct URLData {
    shortlink: String,
    longlink: String,
    hits: i64,
    expiry_time: i64,
}

#[derive(Deserialize)]
struct CreatedURL {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    shorturl: String,
    #[serde(default)]
    longurl: String,
}

async fn setup(
    slug_style: Option<String>,
    test: &str,
) -> (
    config::Config,
    impl Service<Request, Response = ServiceResponse, Error = Error>,
) {
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
    let conf = config::Config {
    port: 4567,
    db_location: format!("/tmp/chhoto-url-test-{test}.sqlite"),
    cache_control_header: None,
    disable_frontend: true,
    site_url: Some(String::from("https://mydomain.com")),
    public_mode: false,
    public_mode_expiry_delay: 0,
    use_temp_redirect: false,
    password: Some(String::from("testpass")),
    hash_algorithm: None,
    api_key: Some(String::from("Z8FNjh2J2v3yfb0xPDIVA58Pj4D0e2jSERVdoqM5pJCbU2w5tmg3PNioD6GUhaQwHHaDLBNZj0EQE8MS4TLKcUyusa05")),
    slug_style: slug_style.unwrap_or(String::from("Pair")),
    slug_length: 8,
    };
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(format!("/tmp/chhoto-url-test-{test}.sqlite")),
                config: conf.clone(),
            }))
            .service(services::siteurl)
            .service(services::version)
            .service(services::add_link)
            .service(services::getall)
            .service(services::delete_link)
            .service(services::link_handler)
            .service(services::delete_link)
            .service(services::expand),
    )
    .await;
    (conf, app)
}

async fn add_link<T: Service<Request, Response = ServiceResponse, Error = Error>, S: Display>(
    app: T,
    api_key: &str,
    shortlink: S,
) -> (StatusCode, CreatedURL) {
    let req = test::TestRequest::post().uri("/api/new")
        .insert_header(("X-API-Key", api_key))
        .set_payload(format!("{{\"shortlink\": \"{shortlink}\", \"longlink\": \"https://example-{shortlink}.com\", \"expiry_delay\": 10}}"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let url: CreatedURL = serde_json::from_str(body.as_str()).unwrap();

    (status, url)
}

//
// The tests start here
//

#[test]
async fn basic_site_config() {
    let test = "basic";
    let (conf, app) = setup(None, test).await;

    let req = test::TestRequest::get().uri("/api/siteurl").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), conf.site_url.unwrap());

    let req = test::TestRequest::get().uri("/api/version").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), env!("CARGO_PKG_VERSION"));

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_shortlink() {
    let test = "adding";
    let (conf, app) = setup(None, test).await;
    let api_key = conf.api_key.unwrap();
    for shortlink in ["test1", "test2", "test3"] {
        let (status, reply) = add_link(&app, &api_key, shortlink).await;
        assert!(status.is_success());
        assert_eq!(reply.shorturl, format!("https://mydomain.com/{shortlink}"));
    }

    let (status, reply) = add_link(&app, &api_key, "test1").await;
    assert!(status.is_client_error());
    assert_eq!(reply.reason, "Short URL is already in use!");
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn link_resolution() {
    let test = "link-resolution";
    let (conf, app) = setup(None, test).await;
    let _ = add_link(&app, &conf.api_key.unwrap(), "test1").await;

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());
    assert_eq!(
        resp.headers().get("location").unwrap(),
        "https://example-test1.com"
    );
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn link_deletion() {
    let test = "link-deletion";
    let (conf, app) = setup(None, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let _ = add_link(&app, &api_key, "test2").await;

    let req = test::TestRequest::delete()
        .uri("/api/del/test2")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn data_fetching_all() {
    let test = "data-fetching-all";
    let (conf, app) = setup(None, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let _ = add_link(&app, &api_key, "test1").await;
    let _ = add_link(&app, &api_key, "test3").await;
    let req = test::TestRequest::get().uri("/test1").to_request();
    let _ = test::call_service(&app, req).await;

    let req = test::TestRequest::get()
        .uri("/api/all")
        .insert_header(("X-API-Key", api_key))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: Vec<URLData> = serde_json::from_str(body.as_str()).unwrap();
    assert_eq!(reply_chunks.len(), 2);
    assert_eq!(reply_chunks[0].shortlink, "test1");
    assert_eq!(reply_chunks[1].shortlink, "test3");
    assert_eq!(reply_chunks[0].longlink, "https://example-test1.com");
    assert_eq!(reply_chunks[1].longlink, "https://example-test3.com");
    assert_eq!(reply_chunks[0].hits, 1);
    assert_eq!(reply_chunks[1].hits, 0);
    assert_ne!(reply_chunks[0].expiry_time, 0);
    assert_ne!(reply_chunks[1].expiry_time, 0);
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_generated_shortlink_with_pair_slug() {
    let test = "shortlink-with-pair-slug";
    let (conf, app) = setup(None, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "").await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z]+-[a-z]+$").unwrap();
    assert!(re.is_match(reply.shorturl.as_str()));
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_generated_shortlink_with_uid_slug() {
    let test = "shortlink-with-uid-slug";
    let (conf, app) = setup(Some(String::from("UID")), test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "").await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z0-9]+$").unwrap();
    assert!(re.is_match(reply.shorturl.as_str()));
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn expand_link() {
    let test = "expand-link";
    let (conf, app) = setup(None, test).await;
    let api_key = conf.api_key.unwrap();
    let _ = add_link(&app, &api_key, "test4").await;

    let req = test::TestRequest::post()
        .uri("/api/expand")
        .insert_header(("X-API-Key", api_key))
        .set_payload("test4")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply: CreatedURL = serde_json::from_str(body.as_str()).unwrap();
    assert_eq!(reply.longurl, "https://example-test4.com");
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}
