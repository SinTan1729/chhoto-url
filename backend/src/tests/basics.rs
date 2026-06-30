// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{body::to_bytes, test};

use super::utils::*;

#[test]
async fn basic_site_config() {
    let test = "basic";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;

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
    assert!(
        body.as_str()
            .starts_with(concat!("Chhoto URL v", env!("CARGO_PKG_VERSION")))
    );

    let req = test::TestRequest::get()
        .uri("/api/getconfig")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = to_bytes(resp.into_body()).await.unwrap();
    let conf: BackendConfig = serde_json::from_str(body.as_str()).unwrap();
    assert!(conf.version.starts_with(env!("CARGO_PKG_VERSION")));
    assert_eq!(conf.slug_length, 8);
}

#[test]
async fn auth_verification() {
    let test = "auth_verification";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;

    let req = test::TestRequest::get().uri("/api/all").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "Unauthorized");

    let status = edit_link(&app, "a", "test2", false, None, None).await;
    assert_eq!(status, 401);

    let (status, reply) = add_link(&app, "a", "test1", 0, "").await;
    assert_eq!(status, 401);
    assert_eq!(reply.reason, "API validation failed.");

    let req = test::TestRequest::delete()
        .uri("/api/del/link")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let req = test::TestRequest::get().uri("/api/getconfig").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}
