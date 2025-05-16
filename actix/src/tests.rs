use actix_http::Request;
use actix_service::Service;
use actix_web::{body::to_bytes, dev::ServiceResponse, test, web::Bytes, App, Error};
use serde::Deserialize;
use std::{env::set_var, fs};

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
    shorturl: String,
}

async fn setup() -> (
    config::Config,
    impl Service<Request, Response = ServiceResponse, Error = Error>,
) {
    set_var("site_url", "https://mydomain.com");
    set_var("api_key", "Z8FNjh2J2v3yfb0xPDIVA58Pj4D0e2jSERVdoqM5pJCbU2w5tmg3PNioD6GUhaQwHHaDLBNZj0EQE8MS4TLKcUyusa053Q8Y3X2o0wbbWdIlU8t5rf9yXjSjAlcGrFSz");
    let conf = config::read();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(String::from("/tmp/chhoto-url-testing-db.sqlite")),
                config: conf.clone(),
            }))
            .service(services::siteurl)
            .service(services::add_link)
            .service(services::getall)
            .service(services::delete_link)
            .service(services::link_handler)
            .service(services::delete_link),
    )
    .await;
    (conf, app)
}

#[test]
async fn test_0_basic_site_config() {
    let (conf, app) = setup().await;

    let req = test::TestRequest::get().uri("/api/siteurl").to_request();

    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), conf.site_url.unwrap());
}

#[test]
async fn test_1_api_adding_link_with_shortlink() {
    let _ = fs::remove_file("/tmp/chhoto-url-testing-db.sqlite");
    let (conf, app) = setup().await;
    for shortlink in ["test1", "test2", "test3"] {
        let req = test::TestRequest::post()
        .uri("/api/new")
        .insert_header(("X-API-Key", conf.api_key.clone().unwrap()))
        .set_payload(format!("{{\"shortlink\": \"{shortlink}\", \"longlink\": \"https://example-{shortlink}.com\", \"expiry_delay\": 10}}"))
        .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = to_bytes(resp.into_body()).await.unwrap();
        let reply_chunks: CreatedURL = serde_json::from_str(body.as_str()).unwrap();
        assert_eq!(
            reply_chunks.shorturl,
            format!("https://mydomain.com/{shortlink}")
        );
    }

    let req = test::TestRequest::post()
    .uri("/api/new")
    .insert_header(("X-API-Key", conf.api_key.clone().unwrap()))
    .set_payload("{\"shortlink\": \"test1\", \"longlink\": \"https://example.com\", \"expiry_delay\": 10}")
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[test]
async fn test_2_api_link_resolution() {
    let (_, app) = setup().await;

    let req = test::TestRequest::get().uri("/test1").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());
    assert_eq!(
        resp.headers().get("location").unwrap(),
        "https://example-test1.com"
    );
}

#[test]
async fn test_3_api_link_deletion() {
    let (conf, app) = setup().await;

    let req = test::TestRequest::delete()
        .uri("/api/del/test2")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[test]
async fn test_4_api_data_fetching_all() {
    let (conf, app) = setup().await;

    let req = test::TestRequest::get()
        .uri("/api/all")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
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
}

#[test]
async fn test_5_api_link_with_generated_shortlink_with_pair_slug() {
    let _ = fs::remove_file("/tmp/chhoto-url-testing-db.sqlite");
    let (conf, app) = setup().await;

    let req = test::TestRequest::post()
        .uri("/api/new")
        .insert_header(("X-API-Key", conf.api_key.clone().unwrap()))
        .set_payload("{\"longlink\": \"https://example.com\", \"expiry_delay\": 10}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: CreatedURL = serde_json::from_str(body.as_str()).unwrap();
    println!("{:#?}", reply_chunks.shorturl);
    assert!(reply_chunks.shorturl.contains("-"));
}

#[test]
async fn test_5_api_link_with_generated_shortlink_with_uid_slug() {
    let _ = fs::remove_file("/tmp/chhoto-url-testing-db.sqlite");
    set_var("slug_style", "UID");
    let (conf, app) = setup().await;

    let req = test::TestRequest::post()
        .uri("/api/new")
        .insert_header(("X-API-Key", conf.api_key.clone().unwrap()))
        .set_payload("{\"longlink\": \"https://example.com\", \"expiry_delay\": 10}")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: CreatedURL = serde_json::from_str(body.as_str()).unwrap();
    println!("{:#?}", reply_chunks.shorturl);
    assert!(!reply_chunks.shorturl.contains("-"));
}
