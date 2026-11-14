//! Versioned JSON request API. No desktop types, paths, or global UI settings.
use crate::feeds;
use serde::Deserialize;
use serde_json::{Value, json};

pub const VERSION: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub args: Arguments,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    #[serde(default)]
    pub sources: Vec<String>,
}

fn error(id: &str, code: &str) -> Value {
    json!({"version": VERSION, "id": id, "ok": false, "error": {"code": code}})
}

pub fn handle(client: &ureq::Agent, input: &[u8]) -> Value {
    if input.len() > MAX_REQUEST_BYTES {
        return error("", "request_too_large");
    }
    let request: Request = match serde_json::from_slice(input) {
        Ok(request) => request,
        Err(_) => return error("", "invalid_request"),
    };
    if request.version != VERSION {
        return error(&request.id, "unsupported_version");
    }
    let data = match request.method.as_str() {
        "health" => json!({"service": "news", "api_version": VERSION}),
        "sources" => Value::Array((0..feeds::SOURCE_COUNT).map(|index| json!({
            "id": feeds::source_id(index), "name": feeds::source_name(index),
            "category": feeds::source_category(index).label(), "subtitle": feeds::source_subtitle(index),
        })).collect()),
        "fetch" => {
            if !(1..=4).contains(&request.args.sources.len()) {
                return error(&request.id, "expected_one_to_four_sources");
            }
            let indices: Option<Vec<_>> = request.args.sources.iter().map(|id|
                (0..feeds::SOURCE_COUNT).find(|&index| feeds::source_id(index) == id)
            ).collect();
            let Some(indices) = indices else { return error(&request.id, "unknown_source"); };
            let rows = std::thread::scope(|scope| {
                let jobs: Vec<_> = indices.into_iter().map(|index| scope.spawn(move || {
                    match feeds::fetch(client, index) {
                        Ok(stories) => json!({"id": feeds::source_id(index), "ok": true,
                            "stories": stories.iter().map(|story| json!({
                                "title": String::from_utf16_lossy(story.title.strip_suffix(&[0]).unwrap_or(&story.title)),
                                "url": String::from_utf16_lossy(story.url.strip_suffix(&[0]).unwrap_or(&story.url)),
                                "badge": story.badge,
                            })).collect::<Vec<_>>() }),
                        Err(_) => json!({"id": feeds::source_id(index), "ok": false, "error": {"code": "source_failed"}}),
                    }
                })).collect();
                jobs.into_iter().map(|job| job.join().expect("news worker panicked")).collect::<Vec<_>>()
            });
            json!(rows)
        }
        _ => return error(&request.id, "unknown_method"),
    };
    json!({"version": VERSION, "id": request.id, "ok": true, "data": data})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_independent_and_has_stable_unique_ids() {
        let result = handle(
            &feeds::client(),
            br#"{"version":1,"id":"catalog","method":"sources"}"#,
        );
        assert_eq!(result["id"], "catalog");
        let rows = result["data"].as_array().unwrap();
        let ids: std::collections::BTreeSet<_> =
            rows.iter().map(|row| row["id"].as_str().unwrap()).collect();
        assert_eq!(ids.len(), feeds::SOURCE_COUNT);
        assert_eq!(rows.len(), feeds::SOURCE_COUNT);
    }

    #[test]
    fn invalid_requests_are_rejected_before_network() {
        let client = feeds::client();
        for (input, code) in [
            (
                r#"{"version":2,"id":"a","method":"fetch"}"#,
                "unsupported_version",
            ),
            (
                r#"{"version":1,"id":"a","method":"fetch","args":{"sources":[]}}"#,
                "expected_one_to_four_sources",
            ),
            (
                r#"{"version":1,"id":"a","method":"fetch","args":{"sources":["not-a-source"]}}"#,
                "unknown_source",
            ),
            (
                r#"{"version":1,"id":"a","method":"delete"}"#,
                "unknown_method",
            ),
            (
                r#"{"version":1,"id":"a","method":"health","unexpected":true}"#,
                "invalid_request",
            ),
        ] {
            assert_eq!(handle(&client, input.as_bytes())["error"]["code"], code);
        }
        assert_eq!(
            handle(&client, &vec![b' '; MAX_REQUEST_BYTES + 1])["error"]["code"],
            "request_too_large"
        );
    }
}
