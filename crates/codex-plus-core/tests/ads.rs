use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use codex_plus_core::ads::{
    DEFAULT_AD_LIST_URLS, cache_busted_ad_url, fetch_ad_list_from_urls, normalize_ad_payload,
};
use serde_json::json;

#[test]
fn default_ad_urls_match_bro_sources() {
    assert_eq!(
        DEFAULT_AD_LIST_URLS,
        [
            "https://api.skuzi.cn/codexplusplus/ads.json",
            "https://api.skuzi.cn/ads.json",
        ]
    );
}

#[test]
fn cache_busted_ad_url_appends_version_query_to_plain_url() {
    assert_eq!(
        cache_busted_ad_url("https://example.test/ads.json", 1779035222758),
        "https://example.test/ads.json?v=1779035222758"
    );
}

#[test]
fn cache_busted_ad_url_preserves_existing_query() {
    assert_eq!(
        cache_busted_ad_url("https://example.test/ads.json?source=cdn", 1779035222758),
        "https://example.test/ads.json?source=cdn&v=1779035222758"
    );
}

#[test]
fn normalizes_remote_ads_for_plugin_and_manager_rendering() {
    let payload = normalize_ad_payload(json!({
        "version": 1,
        "ads": [
            {
                "id": "sponsor",
                "type": "sponsor",
                "title": "赞助商",
                "description": "推荐内容",
                "url": "https://example.test",
                "highlights": ["稳定"]
            },
            {
                "id": "normal",
                "type": "normal",
                "title": "普通推荐",
                "description": "推荐内容",
                "url": "https://example.org"
            },
            {
                "id": "broken",
                "type": "normal",
                "title": "",
                "description": "missing title",
                "url": "https://example.invalid"
            }
        ]
    }));

    assert_eq!(payload["version"], json!(1));
    assert_eq!(payload["ads"].as_array().unwrap().len(), 3);
    assert_eq!(payload["ads"][0]["type"], json!("sponsor"));
    assert_eq!(payload["ads"][1]["id"], json!("bro-api"));
    assert_eq!(payload["ads"][1]["type"], json!("sponsor"));
    assert_eq!(payload["ads"][2]["type"], json!("normal"));
}

#[test]
fn builtin_sponsor_is_appended_after_remote_sponsors() {
    let payload = normalize_ad_payload(json!({
        "version": 1,
        "ads": [
            {
                "id": "remote-sponsor",
                "type": "sponsor",
                "title": "远端赞助商",
                "description": "远端推荐内容",
                "url": "https://example.test"
            },
            {
                "id": "remote-normal",
                "type": "normal",
                "title": "普通推荐",
                "description": "普通推荐内容",
                "url": "https://example.org"
            }
        ]
    }));
    let ads = payload["ads"].as_array().unwrap();

    assert_eq!(ads[0]["id"], json!("remote-sponsor"));
    assert_eq!(ads[1]["id"], json!("bro-api"));
    assert_eq!(ads[1]["title"], json!("BRO API 中转站"));
    assert_eq!(
        ads[1]["url"],
        json!("https://api.skuzi.cn/")
    );
    assert_eq!(ads[1]["expires_at"], json!("2026-08-02T23:59:59+08:00"));
    assert!(
        ads[1]["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    assert_eq!(ads[2]["id"], json!("remote-normal"));
}

#[test]
fn normalizes_bro_sponsor_with_local_logo() {
    let payload = normalize_ad_payload(json!({
        "version": 1,
        "ads": [
            {
                "id": "bro-api",
                "type": "sponsor",
                "title": "BRO API 中转站",
                "description": "远端推荐内容",
                "url": "https://example.test/bro-api"
            }
        ]
    }));
    let ads = payload["ads"].as_array().unwrap();
    let bro_api = ads
        .iter()
        .find(|ad| ad["id"] == json!("bro-api"))
        .unwrap();
    assert!(
        bro_api["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[tokio::test]
async fn fetch_ad_list_tries_backup_url_when_primary_fails() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let thread = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0, "client closed before sending complete headers");
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() <= 16 * 1024, "request headers are too large");
            }
            let request = String::from_utf8_lossy(&request);
            if request.starts_with("GET /primary.json?") {
                stream
                    .write_all(
                        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .unwrap();
            } else {
                assert!(request.starts_with("GET /backup.json?"), "{request}");
                let body = json!({
                    "version": 1,
                    "ads": [{
                        "id": "backup-ad",
                        "type": "normal",
                        "title": "Backup",
                        "description": "Loaded from backup",
                        "url": "https://example.test",
                        "highlights": []
                    }]
                })
                .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
            stream.flush().unwrap();
        }
    });

    let payload = fetch_ad_list_from_urls(&[
        format!("http://127.0.0.1:{port}/primary.json"),
        format!("http://127.0.0.1:{port}/backup.json"),
    ])
    .await
    .unwrap();
    thread.join().unwrap();

    let ads = payload["ads"].as_array().unwrap();
    assert!(ads.iter().any(|ad| ad["id"] == json!("bro-api")));
    assert!(ads.iter().any(|ad| ad["id"] == json!("backup-ad")));
}


