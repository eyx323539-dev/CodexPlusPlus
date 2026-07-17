use serde_json::{Map, Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

const BRO_API_IMAGE: &[u8] = include_bytes!("../../../docs/images/sponsor-bro-api.png");
const BUILTIN_SPONSOR_EXPIRES_AT: &str = "2026-08-02T23:59:59+08:00";

pub const DEFAULT_AD_LIST_URLS: [&str; 2] = [
    "https://api.skuzi.cn/codexplusplus/ads.json",
    "https://api.skuzi.cn/ads.json",
];

pub fn normalize_ad_payload(payload: Value) -> Value {
    let version = payload.get("version").and_then(Value::as_u64).unwrap_or(1);
    let mut ads = payload
        .get("ads")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|ad| {
            let ad_type = ad.get("type").and_then(Value::as_str);
            let title = ad.get("title").and_then(Value::as_str);
            let description = ad.get("description").and_then(Value::as_str);
            let url = ad.get("url").and_then(Value::as_str);
            matches!(ad_type, Some("sponsor" | "normal"))
                && title.is_some_and(|value| !value.trim().is_empty())
                && description.is_some_and(|value| !value.trim().is_empty())
                && url.is_some_and(|value| !value.trim().is_empty())
        })
        .cloned()
        .collect::<Vec<_>>();
    fill_known_remote_logos(&mut ads);
    append_builtin_sponsors(&mut ads);
    json!({ "version": version, "ads": ads })
}

fn fill_known_remote_logos(ads: &mut [Value]) {
    for ad in ads {
        let Some(object) = ad.as_object_mut() else {
            continue;
        };
        let has_image = object
            .get("image")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
        if has_image {
            continue;
        }
        let Some(id) = object.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some((mime, image)) = known_remote_logo(id) else {
            continue;
        };
        object.insert("image".to_string(), json!(data_uri(mime, image)));
    }
}

fn known_remote_logo(id: &str) -> Option<(&'static str, &'static [u8])> {
    match id {
        "bro-api" => Some(("image/png", BRO_API_IMAGE)),
        _ => None,
    }
}

fn append_builtin_sponsors(ads: &mut Vec<Value>) {
    let insert_at = ads
        .iter()
        .rposition(|ad| ad.get("type").and_then(Value::as_str) == Some("sponsor"))
        .map(|index| index + 1)
        .unwrap_or(0);
    let builtins = [builtin_sponsor(
        "bro-api",
        "BRO API 中转站",
        "BRO API 提供稳定、便捷的 AI API 中转服务，支持 Codex、GPT、Claude、Gemini 等常用模型，欢迎开发者注册体验。",
        "https://api.skuzi.cn/",
        BRO_API_IMAGE,
        &["Codex / GPT", "Claude / Gemini", "稳定 API 中转"],
    )];
    let mut cursor = insert_at;
    for sponsor in builtins {
        let id = sponsor.get("id").and_then(Value::as_str);
        if id.is_some_and(|id| {
            ads.iter()
                .any(|ad| ad.get("id").and_then(Value::as_str) == Some(id))
        }) {
            continue;
        }
        ads.insert(cursor, sponsor);
        cursor += 1;
    }
}

fn builtin_sponsor(
    id: &str,
    title: &str,
    description: &str,
    url: &str,
    image: &[u8],
    highlights: &[&str],
) -> Value {
    let mut sponsor = Map::new();
    sponsor.insert("id".to_string(), json!(id));
    sponsor.insert("type".to_string(), json!("sponsor"));
    sponsor.insert("title".to_string(), json!(title));
    sponsor.insert("description".to_string(), json!(description));
    sponsor.insert("url".to_string(), json!(url));
    sponsor.insert("expires_at".to_string(), json!(BUILTIN_SPONSOR_EXPIRES_AT));
    sponsor.insert("image".to_string(), json!(data_uri("image/png", image)));
    sponsor.insert("highlights".to_string(), json!(highlights));
    Value::Object(sponsor)
}

fn data_uri(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64_encode(bytes))
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

pub async fn fetch_ad_list() -> anyhow::Result<Value> {
    // Keep the published build independent from the upstream ad feed. The local
    // sponsor is normalized through the same path as remote payloads.
    Ok(normalize_ad_payload(json!({ "version": 1, "ads": [] })))
}

pub fn cache_busted_ad_url(url: &str, version: u128) -> String {
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}v={version}")
}

pub async fn fetch_ad_list_from_urls<S>(urls: &[S]) -> anyhow::Result<Value>
where
    S: AsRef<str>,
{
    let client = crate::http_client::proxied_client("CodexPlusPlus")?;
    let cache_bust = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let mut last_error = None;
    for url in urls {
        let url = cache_busted_ad_url(url.as_ref(), cache_bust);
        let result = async {
            let response = client.get(url).send().await?.error_for_status()?;
            let payload = response.json::<Value>().await?;
            Ok::<_, anyhow::Error>(normalize_ad_payload(payload))
        }
        .await;
        match result {
            Ok(payload) => return Ok(payload),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("ad list unavailable")))
}
