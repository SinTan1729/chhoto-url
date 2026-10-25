// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{body::to_bytes, mime::APPLICATION_JSON, test};

use crate::config::Credential;

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
        .insert_header(("Accept", APPLICATION_JSON))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body.as_str(),
        r#"{"admin":false,"public_mode_enabled":false,"password_enabled":true,"oidc_enabled":false}"#
    );
    let req = test::TestRequest::get()
        .uri("/api/whoami")
        .insert_header((API_HEADER, conf.api_key.clone().unwrap()))
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
        .insert_header((API_HEADER, conf.api_key.unwrap()))
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

    let req = test::TestRequest::get()
        .uri("/api/all")
        .insert_header((API_HEADER, "random-stuff"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "Wrong API key.");

    let status = edit_link(
        &app,
        &Credential::Plain("a".to_string()),
        "test2",
        false,
        None,
        None,
    )
    .await;
    assert_eq!(status, 401);

    let (status, reply) = add_link(&app, &Credential::Plain("a".to_string()), "test1", 0, "").await;
    assert_eq!(status, 401);
    assert_eq!(reply.reason, "Wrong API key.");

    let req = test::TestRequest::delete()
        .uri("/api/del/link")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let req = test::TestRequest::get().uri("/api/getconfig").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[test]
async fn auth_states() {
    let test = "auth_verification";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;

    let req = test::TestRequest::get().uri("/oidc/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
    let body = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body.as_str(), "OIDC is disabled.");
}
