pub mod domain;
pub mod network;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};

const ID: &str = "com.tezbar.native.ipapi-is";
const COMMAND: &str = "lookup-ip-or-asn";
#[derive(Clone)]
enum Effect {
    Fetch,
    Copy(String),
    Open(String),
}
#[derive(Clone)]
struct Session {
    owner: u64,
    input: String,
    data: Option<Value>,
    error: Option<String>,
    actions: Vec<(String, String, Effect)>,
    uncertain: Option<Effect>,
}
type Sessions = Arc<Mutex<HashMap<String, Session>>>;
struct Guard {
    sessions: Sessions,
    id: String,
    owner: u64,
}
impl Drop for Guard {
    fn drop(&mut self) {
        let mut s = self.sessions.lock().unwrap();
        if s.get(&self.id).is_some_and(|s| s.owner == self.owner) {
            s.remove(&self.id);
        }
    }
}
#[derive(Clone)]
struct Cache {
    profile: PathBuf,
    preferences: Value,
    input: String,
    at: Instant,
    data: Value,
}
pub struct Adapter {
    client: reqwest::Client,
    sessions: Sessions,
    sequence: AtomicU64,
    cache: Mutex<Vec<Cache>>,
}
impl Adapter {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: network::client()?,
            sessions: Arc::default(),
            sequence: AtomicU64::new(1),
            cache: Mutex::new(Vec::new()),
        })
    }
    #[cfg(feature = "qa")]
    #[doc(hidden)]
    pub fn qa_client(client: reqwest::Client) -> Self {
        Self {
            client,
            sessions: Arc::default(),
            sequence: AtomicU64::new(1),
            cache: Mutex::new(Vec::new()),
        }
    }
    fn next(&self) -> Result<u64, String> {
        self.sequence
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| v.checked_add(1))
            .map_err(|_| "Revision capacity exhausted".into())
    }
    fn cached(&self, ctx: &ExtensionContext, prefs: &Value, input: &str) -> Option<Value> {
        let mut c = self.cache.lock().unwrap();
        c.retain(|c| c.at.elapsed() < Duration::from_secs(60));
        c.iter()
            .find(|c| {
                c.profile == ctx.paths.data_dir && &c.preferences == prefs && c.input == input
            })
            .map(|c| c.data.clone())
    }
    fn cache(&self, ctx: &ExtensionContext, prefs: &Value, input: &str, data: Value) {
        let mut c = self.cache.lock().unwrap();
        c.retain(|c| {
            c.at.elapsed() < Duration::from_secs(60)
                && !(c.profile == ctx.paths.data_dir && &c.preferences == prefs && c.input == input)
        });
        if c.len() >= 32 {
            c.remove(0);
        }
        c.push(Cache {
            profile: ctx.paths.data_dir.clone(),
            preferences: prefs.clone(),
            input: input.into(),
            at: Instant::now(),
            data,
        });
    }
    fn render(&self, id: &str) -> CoreResult {
        let mut all = self.sessions.lock().unwrap();
        let s = all.get_mut(id).ok_or("Session ended")?;
        let revision = self.next()?;
        let mut metadata = json!({"type":"Detail.Metadata","children":[]});
        let mut links = Vec::new();
        let mut markdown = domain::heading(&s.input);
        if let Some(d) = &s.data {
            let m = domain::metadata(d);
            metadata["children"] = json!(m.rows);
            links = m.links;
            if d.get("error").is_some() {
                markdown.push_str("Error: The service reported an error. Revalidate to retry.");
            }
        }
        if let Some(e) = &s.error {
            markdown.push_str(&format!("\n{e}"));
        }
        let mut effects = Vec::new();
        if let Some(e) = s.uncertain.clone() {
            markdown.push_str("\nThe host effect was not confirmed. It may already have happened. Retry explicitly to send the exact same bytes.");
            effects.push((
                match e {
                    Effect::Copy(_) => "Retry Exact Copy",
                    Effect::Open(_) => "Retry Exact Open",
                    Effect::Fetch => unreachable!(),
                }
                .into(),
                e,
            ));
        } else {
            if let Some(d) = &s.data {
                effects.push((
                    "Copy All as JSON".into(),
                    Effect::Copy(domain::stringify(d)),
                ));
            }
            effects.push((
                if s.error.is_some() {
                    "Retry IP or ASN"
                } else {
                    "Revalidate IP or ASN"
                }
                .into(),
                Effect::Fetch,
            ));
            effects.extend(links.into_iter().map(|(t, u)| (t, Effect::Open(u))));
        }
        s.actions = effects
            .into_iter()
            .enumerate()
            .map(|(i, (t, e))| (format!("{revision}:{i}"), t, e))
            .collect();
        let actions=s.actions.iter().map(|(id,t,e)|json!({"id":id,"title":t,"kind":match e {Effect::Copy(_)=>"copy",Effect::Open(_)=>"open",Effect::Fetch=>"action"}})).collect();
        let mut v = tezbar_extension_sdk::view(
            "Lookup IP or ASN",
            json!({"type":"Detail","props":{"markdown":markdown,"isLoading":false},"metadata":metadata}),
            actions,
        );
        v["refreshMs"] = Value::Null;
        if s.uncertain.is_some() {
            v["message"]=json!("Host effect failed to confirm; it may already have happened. Retry explicitly with the same bytes.");
        }
        Ok(v)
    }
    async fn fetch(&self, ctx: &ExtensionContext, id: &str, prefs: &Value) -> CoreResult {
        let input = {
            let mut s = self.sessions.lock().unwrap();
            let s = s.get_mut(id).ok_or("Session ended")?;
            s.actions.clear();
            s.input.clone()
        };
        let result = network::lookup(&self.client, &input).await;
        let mut data = None;
        let mut error = None;
        match result {
            Ok(mut d) => {
                if d.get("error").is_some() {
                    d["error"] = json!("The service reported an error. Revalidate to retry.");
                }
                self.cache(ctx, prefs, &input, d.clone());
                data = Some(d);
            }
            Err(e) => {
                error = Some(e);
                if let Some(d) = self.cached(ctx, prefs, &input) {
                    data = Some(d);
                    error=Some("Request failed. Showing a cached response less than 60 seconds old. Revalidate to retry.".into());
                }
            }
        }
        {
            let mut s = self.sessions.lock().unwrap();
            let s = s.get_mut(id).ok_or("Session ended")?;
            s.data = data;
            s.error = error;
            s.uncertain = None;
        }
        self.render(id)
    }
}
// Exact selected host clipboard/shell contract: closed acknowledgement; never null/boolean.
pub fn acknowledgement(v: &Value) -> bool {
    v.as_object().is_some_and(|o| {
        o.get("ok") == Some(&Value::Bool(true))
            && o.keys().all(|k| k == "ok" || k == "message")
            && o.get("message").is_none_or(Value::is_string)
    })
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        ID
    }
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    fn commands(&self) -> &'static [&'static str] {
        &[COMMAND]
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            if command != COMMAND {
                return Err("Unknown command".into());
            }
            let id = payload["sessionId"].as_str().ok_or("Missing session")?;
            match operation {
                "extension:run-command" => {
                    let input = match payload["argumentValues"].get("ip") {
                        None => "",
                        Some(Value::String(s)) => s,
                        Some(_) => return Err("IP or ASN argument must be text".into()),
                    };
                    let owner = self.next()?;
                    {
                        let mut s = self.sessions.lock().unwrap();
                        if s.len() >= 128 || s.contains_key(id) {
                            return Err("Session capacity reached".into());
                        }
                        s.insert(
                            id.into(),
                            Session {
                                owner,
                                input: input.into(),
                                data: None,
                                error: None,
                                actions: Vec::new(),
                                uncertain: None,
                            },
                        );
                    }
                    let guard = Guard {
                        sessions: self.sessions.clone(),
                        id: id.into(),
                        owner,
                    };
                    ctx.spawn(async move {
                        let _guard = guard;
                        std::future::pending::<()>().await;
                    })?;
                    *state = json!({"owner":owner});
                    self.fetch(ctx, id, &payload["preferences"]).await
                }
                "extension:invoke-action" => {
                    let action = payload["actionId"].as_str().ok_or("Missing action")?;
                    let e = {
                        let mut all = self.sessions.lock().unwrap();
                        let s = all.get_mut(id).ok_or("Session ended")?;
                        if state["owner"] != json!(s.owner) {
                            return Err("Session ownership changed".into());
                        }
                        let e = s
                            .actions
                            .iter()
                            .find(|(a, _, _)| a == action)
                            .map(|(_, _, e)| e.clone())
                            .ok_or("Stale or consumed action")?;
                        // Consume before await, independently of SDK transactional state rollback.
                        s.actions.clear();
                        if !matches!(e, Effect::Fetch) {
                            s.uncertain = Some(e.clone());
                        }
                        e
                    };
                    match e {
                        Effect::Fetch => self.fetch(ctx, id, &payload["preferences"]).await,
                        Effect::Copy(text) => {
                            let result = tokio::time::timeout(
                                Duration::from_secs(8),
                                ctx.host("host.clipboard.write", json!({"text":text})),
                            )
                            .await
                            .unwrap_or_else(|_| Err("Host effect timed out".into()));
                            if result.as_ref().is_ok_and(acknowledgement) {
                                self.sessions
                                    .lock()
                                    .unwrap()
                                    .get_mut(id)
                                    .ok_or("Session ended")?
                                    .uncertain = None;
                            }
                            self.render(id)
                        }
                        Effect::Open(target) => {
                            if domain::safe_link(&target).is_none() {
                                return Err("Invalid destination".into());
                            }
                            let result = tokio::time::timeout(
                                Duration::from_secs(8),
                                ctx.host("host.shell.open", json!({"target":target})),
                            )
                            .await
                            .unwrap_or_else(|_| Err("Host effect timed out".into()));
                            if result.as_ref().is_ok_and(acknowledgement) {
                                self.sessions
                                    .lock()
                                    .unwrap()
                                    .get_mut(id)
                                    .ok_or("Session ended")?
                                    .uncertain = None;
                            }
                            self.render(id)
                        }
                    }
                }
                "extension:refresh-session" => {
                    // Recovery after cancellation reads authoritative uncertainty; no HTTP/effect replay.
                    let needs = self
                        .sessions
                        .lock()
                        .unwrap()
                        .get(id)
                        .is_some_and(|s| s.actions.is_empty());
                    if needs {
                        self.render(id)
                    } else {
                        Ok(tezbar_extension_sdk::unchanged())
                    }
                }
                "extension:search-text-changed" | "extension:load-more" => {
                    Ok(tezbar_extension_sdk::unchanged())
                }
                "extension:dispose-session" => {
                    self.sessions.lock().unwrap().remove(id);
                    Ok(json!({}))
                }
                _ => Err("Unsupported operation".into()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tezbar_extension_sdk::{ExtensionPaths, TestContext};
    fn context(profile: &str) -> TestContext {
        TestContext::new(
            ExtensionPaths {
                data_dir: PathBuf::from(profile),
                ..Default::default()
            },
            &["clipboard.write", "shell.open"],
        )
    }
    #[test]
    fn cache_is_profile_preferences_input_scoped_bounded_and_expires() {
        let a = Adapter::new().unwrap();
        let x = context("profile-a");
        let y = context("profile-b");
        a.cache(
            &x.context,
            &json!({"unusedIdentity":"a"}),
            "192.0.2.1",
            json!({"ip":"192.0.2.1","elapsed_ms":0}),
        );
        assert!(a
            .cached(&x.context, &json!({"unusedIdentity":"a"}), "192.0.2.1")
            .is_some());
        assert!(a
            .cached(&y.context, &json!({"unusedIdentity":"a"}), "192.0.2.1")
            .is_none());
        assert!(a
            .cached(&x.context, &json!({"unusedIdentity":"b"}), "192.0.2.1")
            .is_none());
        assert!(a
            .cached(&x.context, &json!({"unusedIdentity":"a"}), "192.0.2.2")
            .is_none());
        a.cache.lock().unwrap()[0].at = Instant::now() - Duration::from_secs(61);
        assert!(a
            .cached(&x.context, &json!({"unusedIdentity":"a"}), "192.0.2.1")
            .is_none());
        for n in 0..40 {
            a.cache(
                &x.context,
                &json!({}),
                &format!("AS{n}"),
                json!({"elapsed_ms":0}),
            );
        }
        assert_eq!(a.cache.lock().unwrap().len(), 32);
    }
}
