//! Per-install service state. The service has no automatic refresh thread.
use crate::{feeds, protocol};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::PathBuf,
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub auto_refresh: bool,
    pub refresh_minutes: u32,
    pub followed: Vec<String>,
    pub hidden: Vec<String>,
    pub order: Vec<String>,
    pub default_followed: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            auto_refresh: true,
            refresh_minutes: 5,
            followed: vec![],
            hidden: vec![],
            order: vec![],
            default_followed: false,
        }
    }
}
impl Preferences {
    fn valid(&self) -> bool {
        (1..=60).contains(&self.refresh_minutes)
            && [&self.followed, &self.hidden, &self.order]
                .iter()
                .all(|ids| {
                    let unique: BTreeSet<_> = ids.iter().collect();
                    unique.len() == ids.len()
                        && ids.iter().all(|id| {
                            (0..feeds::SOURCE_COUNT).any(|index| feeds::source_id(index) == id)
                        })
                })
    }
}
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    preferences: Preferences,
    cache: BTreeMap<String, Value>,
}

pub struct Service {
    path: Option<PathBuf>,
    saved: Saved,
    valid: bool,
}
impl Service {
    pub fn new(path: Option<PathBuf>) -> Self {
        let mut service = Self {
            path,
            saved: Saved::default(),
            valid: true,
        };
        if let Some(path) = &service.path {
            let read = fs::metadata(path).and_then(|metadata| {
                if metadata.len() > 8 * 1024 * 1024 {
                    Err(io::Error::other("news state too large"))
                } else {
                    fs::read(path)
                }
            });
            match read {
                Ok(bytes) => match serde_json::from_slice::<Saved>(&bytes) {
                    Ok(saved) if bytes.len() <= 8 * 1024 * 1024 && saved.preferences.valid() => {
                        service.saved = saved
                    }
                    _ => service.valid = false,
                },
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => service.valid = false,
            }
        }
        service
    }

    pub fn default_path() -> Option<PathBuf> {
        std::env::var_os("LAZYBUDDY_NEWS_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("LAZYBUDDY_DATA_DIR")
                    .map(|path| PathBuf::from(path).join("services/news"))
            })
            .or_else(|| {
                std::env::var_os("LOCALAPPDATA")
                    .map(|path| PathBuf::from(path).join("LazyBuddyNews"))
            })
            .map(|root| root.join("state.json"))
    }

    fn save(&self) -> io::Result<()> {
        if !self.valid {
            return Err(io::Error::other("existing news state is unreadable"));
        }
        let Some(path) = &self.path else {
            return Ok(());
        };
        fs::create_dir_all(path.parent().expect("service state directory"))?;
        let temporary = path.with_extension("tmp");
        let mut file = fs::File::create(&temporary)?;
        serde_json::to_writer(&mut file, &self.saved)?;
        file.sync_all()?;
        drop(file);
        fs::rename(temporary, path)
    }
    fn remember(&mut self, reply: &mut Value, updated_ms: u64) {
        for row in reply["data"]
            .as_array_mut()
            .into_iter()
            .flatten()
            .filter(|row| row["ok"] == true)
        {
            if let Some(id) = row["id"].as_str().map(str::to_owned) {
                row["updated"] = json!(updated_ms / 1000);
                row["updated_ms"] = json!(updated_ms);
                self.saved.cache.insert(id, row.clone());
            }
        }
    }

    pub fn handle(&mut self, client: &ureq::Agent, bytes: &[u8]) -> Value {
        let request = serde_json::from_slice::<Value>(bytes).unwrap_or_default();
        let id = request["id"].as_str().unwrap_or("");
        let error = |code: &str| json!({"version":1,"id":id,"ok":false,"error":{"code":code}});
        let method = request["method"].as_str().unwrap_or("");
        if method == "fetch" && !self.valid {
            return error("state_unreadable");
        }
        if matches!(method, "state" | "preferences" | "configure") {
            if bytes.len() > protocol::MAX_REQUEST_BYTES
                || request["version"] != 1
                || !request["id"].is_string()
                || request.as_object().is_none_or(|root| {
                    root.keys()
                        .any(|key| !matches!(key.as_str(), "version" | "id" | "method" | "args"))
                })
            {
                return error("invalid_request");
            }
            if !self.valid {
                return error("state_unreadable");
            }
            if method == "configure" {
                let Ok(preferences) =
                    serde_json::from_value::<Preferences>(request["args"]["preferences"].clone())
                else {
                    return error("invalid_preferences");
                };
                if !preferences.valid()
                    || request["args"]
                        .as_object()
                        .is_none_or(|args| args.len() != 1)
                {
                    return error("invalid_preferences");
                }
                let previous = self.saved.preferences.clone();
                self.saved.preferences = preferences;
                if self.save().is_err() {
                    self.saved.preferences = previous;
                    return error("state_save_failed");
                }
            } else if request
                .get("args")
                .is_some_and(|args| args.as_object().is_none_or(|args| !args.is_empty()))
            {
                return error("invalid_request");
            }
            let data = if method == "state" {
                serde_json::to_value(&self.saved).expect("news state serializable")
            } else {
                json!({"preferences":self.saved.preferences})
            };
            return json!({"version":1,"id":id,"ok":true,"data":data});
        }
        let mut reply = protocol::handle(client, bytes);
        if method == "fetch" && reply["ok"] == true {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.remember(&mut reply, stamp);
            if self.save().is_err() {
                reply["warning"] = json!("state_save_failed");
            }
        }
        reply
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_fetches_report_the_same_freshness_as_the_saved_cache() {
        let mut service = Service::new(None);
        service.saved.cache.insert(
            "second".into(),
            json!({"id":"second","ok":true,"stories":["old"]}),
        );
        let mut reply =
            json!({"data":[{"id":"first","ok":true,"stories":[]},{"id":"second","ok":false}]});
        service.remember(&mut reply, 1234567);
        assert_eq!(reply["data"][0]["updated_ms"], 1234567);
        assert_eq!(service.saved.cache["first"]["updated_ms"], 1234567);
        assert_eq!(service.saved.cache["second"]["stories"], json!(["old"]));
    }
    #[test]
    fn preferences_validate_sources_and_persist_without_network() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!("qa-news-state-{}", std::process::id()));
        let path = root.join("state.json");
        let mut service = Service::new(Some(path.clone()));
        let client = feeds::client();
        let request = |preferences: Value| {
            serde_json::to_vec(&json!({"version":1,"id":"settings","method":"configure","args":{"preferences":preferences}})).unwrap()
        };
        assert_eq!(
            service.handle(&client, &request(json!({"refresh_minutes":0})))["ok"],
            false
        );
        assert_eq!(
            service.handle(&client, &request(json!({"followed":["missing"]})))["ok"],
            false
        );
        assert_eq!(
            service.handle(&client, &request(json!({"refresh_minutes":10})))["ok"],
            true
        );
        let mut restored = Service::new(Some(path.clone()));
        assert_eq!(
            restored.handle(&client, br#"{"version":1,"id":"restore","method":"state"}"#)["data"]["preferences"]
                ["refresh_minutes"],
            10
        );
        fs::write(&path, b"broken").unwrap();
        assert_eq!(
            Service::new(Some(path.clone())).handle(&client, &request(json!({})))["error"]["code"],
            "state_unreadable"
        );
        assert_eq!(
            Service::new(Some(path.clone())).handle(
                &client,
                br#"{"version":1,"id":"refresh","method":"fetch","args":{"sources":["zhihu"]}}"#
            )["error"]["code"],
            "state_unreadable"
        );
        assert_eq!(fs::read(path).unwrap(), b"broken");
        fs::remove_dir_all(root).unwrap();
    }
}
