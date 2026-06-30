// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

use actix_web::{body::to_bytes, test};
use regex::Regex;
use std::{thread::sleep, time::Duration};

use super::utils::*;
use crate::*;

#[test]
async fn adding_link_with_shortlink() {
    let test = "adding";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    for shortlink in ["test1", "test2", "test3"] {
        let (status, reply) = add_link(&app, &api_key, shortlink, 10, "").await;
        assert!(status.is_success());
        assert_eq!(reply.shortlink, format!("https://mydomain.com/{shortlink}"));
    }

    let (status, reply) = add_link(&app, &api_key, "test1", 10, "").await;
    assert!(status.is_client_error());
    assert_eq!(reply.reason, "Short URL is already in use!");
}

#[test]
async fn adding_link_with_shortlink_capital_letters() {
    let test = "adding-capital";
    let mut conf = default_config(test);
    conf.allow_capital_letters = true;
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.unwrap();
    for shortlink in ["Test1", "Test2", "Test3"] {
        let (status, reply) = add_link(&app, &api_key, shortlink, 10, "").await;
        assert!(status.is_success());
        assert_eq!(reply.shortlink, format!("https://mydomain.com/{shortlink}"));
    }

    let (status, reply) = add_link(&app, &api_key, "Test1", 10, "").await;
    assert!(status.is_client_error());
    assert_eq!(reply.reason, "Short URL is already in use!");
}

#[test]
async fn adding_link_with_generated_shortlink_with_pair_slug() {
    let test = "shortlink-with-pair-slug";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10, "").await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z]+-[a-z]+$").unwrap();
    assert!(re.is_match(reply.shortlink.as_str()));
}

#[test]
async fn adding_link_with_generated_shortlink_with_uid_slug() {
    let test = "autogen-with-uid-slug";
    let mut conf = default_config(test);
    conf.slug_style = config::SlugStyle::Uid;
    conf.slug_length = 12;
    let (_tempdir, app) = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10, "").await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[a-z0-9]{12}$").unwrap();
    assert!(re.is_match(reply.shortlink.as_str()));
}

#[test]
async fn batch_insertion() {
    let test = "batch-insertion";
    let mut conf = default_config(test);
    conf.slug_style = config::SlugStyle::Uid;
    conf.slug_length = 12;
    let (_tempdir, app) = create_app(&conf, test).await;
    let req = test::TestRequest::post()
        .uri("/api/new")
        .insert_header(("X-API-Key", conf.api_key.unwrap()))
        .set_payload(
            r#"[{"shortlink":"test1","longlink":"https://example.com/test1"},
        {"shortlink":"test2","longlink":"https://example.com/test2"},
        {"longlink":"https://example.com/test2", "expiry_delay": 10},
        {"shortlink":"test1","longlink":"https://example.com/test3"}]"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = to_bytes(resp.into_body()).await.unwrap();
    let mut urls: Vec<URLData> = serde_json::from_str(body.as_str()).unwrap();

    assert!(status.is_success());
    assert_eq!(urls.pop().unwrap().reason, "Short URL is already in use!");
    assert!(urls.pop().unwrap().expiry_time > 0);
    assert_eq!(urls.pop().unwrap().shortlink, "https://mydomain.com/test2");
    assert_eq!(urls.pop().unwrap().shortlink, "https://mydomain.com/test1");
}

#[test]
async fn adding_link_with_generated_shortlink_with_uid_slug_capital_letters() {
    let test = "autogen-with-uid-slug-capital";
    let mut conf = default_config(test);
    conf.slug_style = config::SlugStyle::Uid;
    conf.slug_length = 12;
    conf.allow_capital_letters = true;
    let (_tempdir, app) = create_app(&conf, test).await;
    let (status, reply) = add_link(&app, &conf.api_key.unwrap(), "", 10, "").await;

    assert!(status.is_success());
    let re = Regex::new(r"^https://mydomain.com/[A-Za-z0-9]{12}$").unwrap();
    assert!(re.is_match(reply.shortlink.as_str()));
}

#[test]
async fn adding_link_with_retry_on_collision() {
    let test = "retry_on_collision";
    let mut conf = default_config(test);
    conf.slug_style = config::SlugStyle::Uid;
    conf.slug_length = 1;
    conf.try_longer_slug = true;

    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = &conf.api_key.unwrap();

    // Add every possible single-character shortlink
    {
        #[rustfmt::skip]
        static CHARS: [char; 36] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
            'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x','y',
            'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

        for c in CHARS.iter() {
            let (status, _) = add_link(&app, api_key, c, 10, "").await;
            assert!(status.is_success());
        }
    }

    // Generated shortlinks should now be 5 characters
    {
        let (status, reply) = add_link(&app, api_key, "", 10, "").await;
        assert!(status.is_success());
        assert_eq!(
            reply.shortlink.chars().count(),
            "https://mydomain.com/".len() + 5
        );
    }

    // But a colliding provided shortlink should fail
    {
        let (status, _) = add_link(&app, api_key, "a", 10, "").await;
        assert!(status.is_client_error());
    }
}

#[test]
async fn link_editing() {
    let test = "link-editing";
    let conf = default_config(test);
    let (_tempdir, app) = create_app(&conf, test).await;
    let api_key = conf.api_key.clone().unwrap();

    let (status, _) = add_link(&app, &api_key, "test1", 0, "").await;
    assert!(status.is_success());
    let (status, _) = add_link(&app, &api_key, "test2", 10, "").await;
    assert!(status.is_success());

    let req = test::TestRequest::get().uri("/test2").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection());

    let timer = Duration::from_millis(600);
    tokio::time::sleep(timer).await;
    let now = chrono::Utc::now().timestamp();
    let status = edit_link(&app, &api_key, "test2", false, Some(now + 1), None).await;
    assert!(status.is_success());

    let (status, reply) = expand(&app, &api_key, "test2").await;
    assert!(status.is_success());
    assert_eq!(reply.longlink, "https://edited-test2.com");
    assert_eq!(reply.hits, 1);
    assert_eq!(reply.expiry_time, now + 1);

    let req = test::TestRequest::get().uri("/test1").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_redirection());
    let status = edit_link(&app, &api_key, "test1", true, None, None).await;
    assert!(status.is_success());

    let (status, reply) = expand(&app, &api_key, "test1").await;
    assert!(status.is_success());
    assert_eq!(reply.longlink, "https://edited-test1.com");
    assert_eq!(reply.hits, 0);

    let one_second = Duration::from_secs(1);
    sleep(one_second);
    let status = edit_link(&app, &api_key, "test2", true, None, None).await;
    assert!(status.is_client_error());
}
