// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::test;
use tokio::time::{Duration, sleep};

use super::utils::*;

#[test]
async fn link_resolution() {
    let test = "link-resolution";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let (status, _) = add_link(&app, &conf.api_key.unwrap(), "test1", 10, "").await;
    assert!(status.is_success());

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());
    assert_eq!(
        resp.headers().get("location").unwrap(),
        "https://example-test1.com"
    );
}

#[test]
async fn link_deletion() {
    let test = "link-deletion";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let (status, _) = add_link(&app, &api_key, "test2", 10, "").await;
    assert!(status.is_success());

    let req = test::TestRequest::delete()
        .uri("/api/del/test2")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[test]
async fn data_fetching_all() {
    let test = "data-fetching-all";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();
    let _ = add_link(&app, &api_key, "test1", 10, "").await;
    let _ = add_link(&app, &api_key, "test3", 10, "").await;
    let req = test::TestRequest::get().uri("/test1").to_request();
    let _ = test::call_service(&app, req).await;

    let timer = Duration::from_millis(800);
    tokio::time::sleep(timer).await;
    let reply = getall(&app, &api_key, "").await;
    assert_eq!(reply.len(), 2);
    assert_eq!(reply[0].shortlink, "test1");
    assert_eq!(reply[1].shortlink, "test3");
    assert_eq!(reply[0].longlink, "https://example-test1.com");
    assert_eq!(reply[1].longlink, "https://example-test3.com");
    assert_eq!(reply[0].hits, 1);
    assert_eq!(reply[1].hits, 0);
    assert_ne!(reply[0].expiry_time, 0);
    assert_ne!(reply[1].expiry_time, 0);

    let reply = getall(&app, &api_key, "page_size=1").await;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].shortlink, "test3");

    let reply = getall(&app, &api_key, "page_no=2&page_size=1").await;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].shortlink, "test1");

    let reply = getall(&app, &api_key, "page_after=test3&page_size=1").await;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].shortlink, "test1");
}

#[test]
async fn expand_link() {
    let test = "expand-link";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    let _ = add_link(&app, &api_key, "test4", 10, "test-note").await;

    let (status, reply) = expand(&app, &api_key, "test4").await;
    assert!(status.is_success());
    assert_eq!(reply.longlink, "https://example-test4.com");
    assert_eq!(reply.notes, "test-note");
}

#[test]
async fn link_expiry() {
    let test = "link-expiry";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 1, "").await;
    assert!(status.is_success());
    let one_second = Duration::from_secs(1);
    sleep(one_second).await;

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let (status, _) = expand(&app, &api_key, "test1").await;
    assert!(status.is_client_error());
    // We should be able to add it again right away
    let (status, _) = add_link(&app, &api_key, "test1", 10, "").await;
    assert!(status.is_success());
}

#[test]
async fn notes_and_filtering() {
    let test = "notes-and-filtering";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 0, "note1").await;
    assert!(status.is_success());
    let (status, _) = add_link(&app, &api_key, "test2", 10, "note2").await;
    assert!(status.is_success());

    let status = edit_link(&app, &api_key, "test2", false, None, Some("changed")).await;
    assert!(status.is_success());

    let reply = getall(&app, &api_key, "filter=chan").await;
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].shortlink, "test2");
    assert_eq!(reply[0].notes, "changed");

    let reply = getall(&app, &api_key, "filter=tes").await;
    assert_eq!(reply.len(), 2);
    assert_eq!(reply[1].shortlink, "test2");
    assert_eq!(reply[0].notes, "note1");
}
#[test]
async fn edit_expiry() {
    let test = "link-editing";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 10, "").await;
    assert!(status.is_success());

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());

    let status = edit_link(&app, &api_key, "test1", false, None, None).await;
    assert!(status.is_success());

    let (status, reply) = expand(&app, &api_key, "test1").await;
    assert!(status.is_success());
    assert_eq!(reply.longlink, "https://edited-test1.com");

    let now = chrono::Utc::now().timestamp();
    let status = edit_link(&app, &api_key, "test1", false, Some(now + 1), None).await;
    assert!(status.is_success());

    let one_second = Duration::from_secs(1);
    sleep(one_second).await;
    let status = edit_link(&app, &api_key, "test1", true, None, None).await;
    assert!(status.is_client_error());
}
