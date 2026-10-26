use serde_json::Value;
use std::collections::HashSet;
use super::process_install;
fn bounded_text<'a>(v: &'a Value, name: &str, max: usize) -> Result<&'a str, String> {
    v[name]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control))
        .ok_or_else(|| format!("Invalid launcher {name}"))
}
pub(super) fn validate(m: &Value) -> Result<(), String> {
    let ds = match m.get("launcherContributions") {
        Some(ds) => ds
            .as_array()
            .filter(|a| a.len() <= 8)
            .ok_or("Too many launcher contributions")?
            .as_slice(),
        None => &[],
    };
    let mut ids = HashSet::new();
    for d in ds {
        let id = bounded_text(d, "id", 128)?;
        process_install::component(id)?;
        if !ids.insert(id) {
            return Err("Duplicate launcher contribution".into());
        }
        let command = bounded_text(d, "commandName", 128)?;
        let cmd = m["commands"]
            .as_array()
            .and_then(|a| a.iter().find(|c| c["name"] == command))
            .ok_or("Launcher must target its own command")?;
        let surfaces = d["surfaces"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 2)
            .ok_or("Invalid launcher surfaces")?;
        if surfaces.iter().any(|s| s != "query" && s != "argument") {
            return Err("Invalid launcher surface".into());
        }
        if surfaces.iter().any(|s| s == "argument") {
            let name = bounded_text(d, "argumentName", 128)?;
            if !cmd["arguments"]
                .as_array()
                .is_some_and(|a| a.iter().any(|a| a["name"] == name))
            {
                return Err("Launcher argument must belong to its command".into());
            }
        }
        if d["limit"].as_u64().is_none_or(|n| !(1..=32).contains(&n))
            || d["timeoutMs"]
                .as_u64()
                .is_none_or(|n| !(50..=3000).contains(&n))
        {
            return Err("Invalid launcher bounds".into());
        }
        let selectors = d["selectors"]
            .as_object()
            .ok_or("Missing launcher selectors")?;
        if selectors
            .keys()
            .any(|k| !["terms", "digitRun"].contains(&k.as_str()))
        {
            return Err("Unknown launcher selector".into());
        }
        if let Some(terms) = selectors.get("terms") {
            let terms = terms
                .as_array()
                .filter(|a| a.len() <= 32)
                .ok_or("Invalid launcher terms")?;
            if terms.iter().any(|t| {
                t.as_str().is_none_or(|s| {
                    s.is_empty() || s.len() > 128 || s.chars().any(char::is_control)
                })
            }) {
                return Err("Invalid launcher term".into());
            }
        }
        if let Some(n) = selectors.get("digitRun") {
            let min = n["min"].as_u64().ok_or("Invalid digit selector")?;
            let max = n["max"].as_u64().ok_or("Invalid digit selector")?;
            if !(1..=16).contains(&min) || !(min..=16).contains(&max) {
                return Err("Invalid digit selector".into());
            }
        }
        if d["index"] == true
            && !m["capabilities"].as_array().is_some_and(|a| {
                a.iter().any(|v| {
                    v == "launcher" || v == "launcher.publish" || v == "host.launcher.publish"
                })
            })
        {
            return Err("Launcher index requires publish capability".into());
        }
        if d.get("index").is_some_and(|v| !v.is_boolean()) {
            return Err("Invalid launcher index policy".into());
        }
        if let Some(paths) = d.get("bootstrapMigrationPaths") {
            if d["index"] != true
                || paths.as_array().is_none_or(|a| {
                    a.is_empty()
                        || a.len() > 8
                        || a.iter().any(|p| {
                            p.as_str()
                                .is_none_or(|k| m["migrationPaths"].get(k).is_none())
                        })
                })
            {
                return Err("Invalid launcher bootstrap grants".into());
            }
        }
    }
    for cmd in m["commands"].as_array().into_iter().flatten() {
        if let Some(l) = cmd.get("launcher") {
            if let Some(id) = l.get("id") {
                let command = cmd["name"].as_str().unwrap_or("");
                let valid = std::iter::once(&m["id"])
                    .chain(m["replaces"].as_array().into_iter().flatten())
                    .filter_map(Value::as_str)
                    .any(|ext| id == &format!("extcmd:{ext}:{command}"));
                if !valid {
                    return Err("Invalid launcher command ID".into());
                }
            }
            if l["recommendation"].as_u64().is_none_or(|n| n > 2000) {
                return Err("Invalid launcher recommendation".into());
            }
            if let Some(a) = l.get("inlineArgument") {
                let name = bounded_text(a, "name", 128)?;
                if !cmd["arguments"]
                    .as_array()
                    .is_some_and(|v| v.iter().any(|v| v["name"] == name))
                {
                    return Err("Invalid inline argument association".into());
                }
                bounded_text(a, "title", 128)?;
                bounded_text(a, "prompt", 256)?;
                if let Some(v) = a.get("validation") {
                    if v["kind"] != "integer"
                        || v["maxLength"]
                            .as_u64()
                            .is_none_or(|n| !(1..=16).contains(&n))
                        || v["min"]
                            .as_u64()
                            .zip(v["max"].as_u64())
                            .is_none_or(|(min, max)| min > max)
                    {
                        return Err("Invalid inline validation".into());
                    }
                    bounded_text(v, "message", 256)?;
                }
            }
        }
    }
    Ok(())
}
