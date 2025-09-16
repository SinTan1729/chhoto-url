use actix_http::{Request, StatusCode};
use actix_service::Service;
use actix_web::test;
use actix_web::{body::to_bytes, dev::ServiceResponse, web::Bytes, App, Error};
use regex::Regex;
use serde::Deserialize;
use std::{fmt::Display, fs, thread::sleep, time::Duration};

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
    #[serde(default)]
    hits: i64,
}

#[derive(Deserialize)]
struct BackendConfig {
    version: String,
    slug_length: usize,
}

fn default_config(test: &str) -> config::Config {
    let conf = config::Config {
    address: String::from("0.0.0.0"),
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
    slug_style: "Pair".to_string(),
    slug_length: 8,
    try_longer_slug: false,
    allow_capital_letters: false,
    custom_landing_directory: None,
    };
    conf
}

async fn create_app(
    conf: &config::Config,
    test: &str,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(format!("/tmp/chhoto-url-test-{test}.sqlite")),
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
    .await;
    app
}

async fn add_link<T: Service<Request, Response = ServiceResponse, Error = Error>, S: Display>(
    app: T,
    api_key: &str,
    shortlink: S,
    expiry_delay: i64,
) -> (StatusCode, CreatedURL) {
    let req = test::TestRequest::post().uri("/api/new")
        .insert_header(("X-API-Key", api_key))
        .set_payload(format!("{{\"shortlink\":\"{shortlink}\",\"longlink\":\"https://example-{shortlink}.com\",\"expiry_delay\":{expiry_delay}}}"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let url: CreatedURL = serde_json::from_str(body.as_str()).unwrap();

    (status, url)
}

async fn expand<T: Service<Request, Response = ServiceResponse, Error = Error>, S: Display>(
    app: T,
    api_key: &str,
    shortlink: S,
) -> (StatusCode, CreatedURL) {
    let req = test::TestRequest::post()
        .uri("/api/expand")
        .insert_header(("X-API-Key", api_key))
        .set_payload(shortlink.to_string())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let url: CreatedURL = serde_json::from_str(body.as_str()).unwrap();

    (status, url)
}

async fn edit_link<T: Service<Request, Response = ServiceResponse, Error = Error>>(
    app: T,
    api_key: &str,
    shortlink: &str,
    reset_hits: bool,
) -> StatusCode {
    let req = test::TestRequest::put()
        .uri("/api/edit")
        .insert_header(("X-API-Key", api_key))
        .set_payload(format!("{{\"shortlink\":\"{shortlink}\",\"longlink\":\"https://edited-{shortlink}.com\",\"reset_hits\":{reset_hits}}}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    resp.status()
}

//
// The tests start here
//

#[test]
async fn basic_site_config() {
    let test = "basic";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;

    let req = test::TestRequest::get().uri("/api/siteurl").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), conf.site_url.unwrap());

    let req = test::TestRequest::get().uri("/api/whoami").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "nobody");
    let req = test::TestRequest::get()
        .uri("/api/whoami")
        .insert_header(("X-API-Key", conf.api_key.clone().unwrap()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "admin");

    let req = test::TestRequest::get().uri("/api/version").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body.as_str(),
        format!("Chhoto URL v{}", env!("CARGO_PKG_VERSION"))
    );

    let req = test::TestRequest::get()
        .uri("/api/getconfig")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let conf: BackendConfig = serde_json::from_str(body.as_str()).unwrap();
    assert_eq!(conf.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(conf.slug_length, 8);

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_shortlink() {
    let test = "adding";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    for shortlink in ["test1", "test2", "test3"] {
        let (status, reply) = add_link(&app, &api_key, shortlink, 10).await;
        assert!(status.is_success());
        assert_eq!(reply.shorturl, format!("https://mydomain.com/{shortlink}"));
    }

    let (status, reply) = add_link(&app, &api_key, "test1", 10).await;
    assert!(status.is_client_error());
    assert_eq!(reply.reason, "Short URL is already in use!");

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_shortlink_capital_letters() {
    let test = "adding-capital";
    let mut conf = default_config(test);
    conf.allow_capital_letters = true;
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    for shortlink in ["Test1", "Test2", "Test3"] {
        let (status, reply) = add_link(&app, &api_key, shortlink, 10).await;
        assert!(status.is_success());
        assert_eq!(reply.shorturl, format!("https://mydomain.com/{shortlink}"));
    }

    let (status, reply) = add_link(&app, &api_key, "Test1", 10).await;
    assert!(status.is_client_error());
    assert_eq!(reply.reason, "Short URL is already in use!");

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn link_resolution() {
    let test = "link-resolution";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let (status, _) = add_link(&app, &conf.api_key.unwrap(), "test1", 10).await;
    assert!(status.is_success());

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
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let (status, _) = add_link(&app, &api_key, "test2", 10).await;
    assert!(status.is_success());

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
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let _ = add_link(&app, &api_key, "test1", 10).await;
    let _ = add_link(&app, &api_key, "test3", 10).await;
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
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10).await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z]+-[a-z]+$").unwrap();
    assert!(re.is_match(reply.shorturl.as_str()));

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_generated_shortlink_with_uid_slug() {
    let test = "autogen-with-uid-slug";
    let mut conf = default_config(test);
    conf.slug_style = "UID".to_string();
    conf.slug_length = 12;
    let app = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10).await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z0-9]{12}$").unwrap();
    assert!(re.is_match(reply.shorturl.as_str()));

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_generated_shortlink_with_uid_slug_capital_letters() {
    let test = "autogen-with-uid-slug-capital";
    let mut conf = default_config(test);
    conf.slug_style = "UID".to_string();
    conf.slug_length = 12;
    conf.allow_capital_letters = true;
    let app = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10).await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[A-Za-z0-9]{12}$").unwrap();
    assert!(re.is_match(reply.shorturl.as_str()));

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn adding_link_with_retry_on_collision() {
    let test = "retry_on_collision";
    let mut conf = default_config(test);
    conf.slug_style = "UID".to_string();
    conf.slug_length = 1;
    conf.try_longer_slug = true;

    let app = create_app(&conf, test).await;
    let api_key = &conf.api_key.unwrap();

    // Add every possible single-character shorturl
    {
        #[rustfmt::skip]
        static CHARS: [char; 36] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

        for c in CHARS.iter() {
            let (status, _) = add_link(&app, api_key, c, 10).await;
            assert!(status.is_success());
        }
    }

    // Generated shorturls should now be 5 characters
    {
        let (status, reply) = add_link(&app, api_key, "", 10).await;
        assert!(status.is_success());
        assert_eq!(
            reply.shorturl.chars().count(),
            "https://mydomain.com/".len() + 5
        );
    }

    // But a colliding provided shorturl should fail
    {
        let (status, _) = add_link(&app, api_key, "a", 10).await;
        assert!(status.is_client_error());
    }

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn expand_link() {
    let test = "expand-link";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    let _ = add_link(&app, &api_key, "test4", 10).await;

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

#[test]
async fn link_expiry() {
    let test = "link-expiry";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 1).await;
    assert!(status.is_success());
    let one_second = Duration::from_secs(1);
    sleep(one_second);

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let (status, _) = expand(&app, &api_key, "test1").await;
    assert!(status.is_client_error());
    // We should be able to add it again right away
    let (status, _) = add_link(&app, &api_key, "test1", 10).await;
    assert!(status.is_success());

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}

#[test]
async fn link_editing() {
    let test = "link-editing";
    let conf = default_config(test);
    let app = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 0).await;
    assert!(status.is_success());
    let (status, _) = add_link(&app, &api_key, "test2", 1).await;
    assert!(status.is_success());

    let req = test::TestRequest::get().uri("/test2").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());

    let status = edit_link(&app, &api_key, "test2", false).await;
    assert!(status.is_success());

    let (status, reply) = expand(&app, &api_key, "test2").await;
    assert!(status.is_success());
    assert_eq!(reply.longurl, "https://edited-test2.com");
    assert_eq!(reply.hits, 1);

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_redirection());
    let status = edit_link(&app, &api_key, "test1", true).await;
    assert!(status.is_success());

    let (status, reply) = expand(&app, &api_key, "test1").await;
    assert!(status.is_success());
    assert_eq!(reply.longurl, "https://edited-test1.com");
    assert_eq!(reply.hits, 0);

    let one_second = Duration::from_secs(1);
    sleep(one_second);
    let status = edit_link(&app, &api_key, "test2", true).await;
    assert!(status.is_client_error());

    let _ = fs::remove_file(format!("/tmp/chhoto-url-test-{test}.sqlite"));
}
