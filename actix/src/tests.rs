use actix_http::Request;
use actix_service::Service;
use actix_web::{body::to_bytes, dev::ServiceResponse, test, web::Bytes, App, Error};
use serde::{Deserialize, Serialize};
use std::env::set_var;

use super::*;

trait BodyTest {
    fn as_str(&self) -> &str;
}

impl BodyTest for Bytes {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

#[derive(Serialize)]
struct _NewURL {
    #[serde(default)]
    shorturl: String,
    longurl: String,
    #[serde(default)]
    expiry_delay: i64,
}

#[derive(Deserialize)]
struct CreatedURL {
    shorturl: String,
}

async fn setup() -> (
    config::Config,
    impl Service<Request, Response = ServiceResponse, Error = Error>,
) {
    set_var("site_url", "https://example.com");
    set_var("api_key", "Z8FNjh2J2v3yfb0xPDIVA58Pj4D0e2jSERVdoqM5pJCbU2w5tmg3PNioD6GUhaQwHHaDLBNZj0EQE8MS4TLKcUyusa053Q8Y3X2o0wbbWdIlU8t5rf9yXjSjAlcGrFSz");
    let conf = config::read();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState {
                db: database::open_db(String::from("/tmp/urls.sqlite")),
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
async fn basic_site_config() {
    let (conf, app) = setup().await;
    // Test siteurl
    let req = test::TestRequest::get().uri("/api/siteurl").to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), conf.site_url.unwrap());
}

#[test]
async fn api_link_operations() {
    let (conf, app) = setup().await;
    // Test url adding
    let req = test::TestRequest::post()
        .uri("/api/new")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .set_payload("{\"shortlink\": \"test1\", \"longlink\": \"https://example.com\", \"expiry_delay\": 10}")
        .to_request();
    println!("{:#?}", req);
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let reply_chunks: CreatedURL = serde_json::from_str(body.as_str()).unwrap();
    assert_eq!(reply_chunks.shorturl, "https://example.com/test1");
}
