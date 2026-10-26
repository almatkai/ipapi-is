use serde_json::Value;
use std::{path::PathBuf,collections::HashSet};
const CAPABILITY_GROUPS: &[&str] = &[
    "clipboard",
    "shell",
    "selection",
    "window",
    "dialog",
    "screen",
    "apps",
    "settings",
    "ai",
    "storage",
    "background",
    "tray",
    "notifications",
    "safety",
    "launcher",
];

/// Closed service allowlist shared by manifest validation and runtime grants.
pub(super) const METHODS: &[&str] = &[
    "host.clipboard.read",
    "host.clipboard.write",
    "host.clipboard.writeImage",
    "host.clipboard.writeFiles",
    "host.clipboard.paste",
    "host.clipboard.pasteText",
    "host.shell.open",
    "host.shell.openWithApp",
    "host.shell.reveal",
    "host.selection.read",
    "host.window.visibility",
    "host.window.suppressBlur",
    "host.dialog.confirm",
    "host.screen.capture",
    "host.apps.list",
    "host.settings.open",
    "host.ai.ask",
    "host.background.retain",
    "host.background.release",
    "host.tray.set",
    "host.tray.remove",
    "host.notifications.show",
    "host.notifications.remove",
    "host.safety.authorize",
    "host.safety.record",
    "host.launcher.publish",
];
fn valid_capability(cap: &str) -> bool {
    CAPABILITY_GROUPS.contains(&cap)
        || METHODS
            .iter()
            .any(|m| *m == cap || m.strip_prefix("host.") == Some(cap))
}

pub(super) fn component(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 200
        || !s.as_bytes()[0].is_ascii_alphanumeric()
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err("Invalid package component".into());
    }
    Ok(())
}
pub(super) fn relative(s: &str) -> Result<PathBuf, String> {
    // Reject cross-platform path syntax even when verifying on Unix.
    if s.is_empty()
        || s.len() > 1024
        || s.contains(['\\', ':', '\0'])
        || s.chars().any(|c| c.is_control())
        || s.starts_with('/')
        || s.split('/')
            .any(|p| p.is_empty() || p == "." || p == ".." || p.ends_with([' ', '.']))
    {
        return Err(format!("Unsafe package path: {s}"));
    }
    for part in s.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ]
        .contains(&stem.as_str())
        {
            return Err("Reserved package filename".into());
        }
    }
    Ok(PathBuf::from(s))
}
pub(super) fn validate(manifest: &Value) -> Result<(), String> {
    if manifest["schemaVersion"] != 3
        || manifest["runtime"] != "rust"
        || manifest["execution"] != "process"
        || manifest["protocolVersion"] != 1
    {
        return Err("Extension needs a schema-3 process package (protocol 1); schema-2 markers need upgrade".into());
    }
    let slug = manifest["slug"].as_str().ok_or("Missing slug")?;
    component(slug)?;
    if manifest["id"] != format!("com.tezbar.native.{slug}") {
        return Err("Package identity mismatch".into());
    }
    let version = manifest["version"].as_str().ok_or("Missing version")?;
    component(version)?;
    if let Some(min) = manifest["minHostVersion"].as_str() {
        let min = semver::Version::parse(min).map_err(|_| "Invalid minimum host version")?;
        let host = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
        if host < min {
            return Err("Package requires a newer Tezbar host".into());
        }
    }
    let caps = manifest["capabilities"]
        .as_array()
        .ok_or("Missing capabilities")?;
    if caps
        .iter()
        .any(|c| !c.as_str().is_some_and(|s| valid_capability(s)))
    {
        return Err("Unknown capability".into());
    }
    let commands = manifest["commands"].as_array().ok_or("Missing commands")?;
    let mut names = HashSet::new();
    for c in commands {
        let name = c["name"].as_str().ok_or("Missing command name")?;
        component(name)?;
        if !names.insert(name) {
            return Err("Duplicate command".into());
        }
        if c.get("concurrency")
            .is_some_and(|v| v != "serialize" && v != "reject-while-busy")
        {
            return Err("Invalid concurrency policy".into());
        }
        if c.get("timeoutMs")
            .is_some_and(|v| !v.as_u64().is_some_and(|n| (1..=300000).contains(&n)))
        {
            return Err("Invalid command deadline".into());
        }
        if c.get("refreshMs")
            .is_some_and(|v| !v.is_null() && !v.as_u64().is_some_and(|n| n >= 500))
        {
            return Err("Invalid refresh interval".into());
        }
    }
    super::launcher::validate(manifest)?;
    if let Some(paths) = manifest.get("migrationPaths") {
        let paths = paths.as_object().ok_or("Invalid migration paths")?;
        if paths.len() > 32 {
            return Err("Too many migration paths".into());
        }
        for (key, path) in paths {
            component(key)?;
            relative(path.as_str().ok_or("Invalid migration path")?)?;
        }
    }
    if let Some(probe) = manifest.get("startupProbe") {
        component(probe["key"].as_str().ok_or("Startup probe requires key")?)?;
        let paths = probe["migrationPaths"]
            .as_array()
            .ok_or("Startup probe requires migration path keys")?;
        if paths.is_empty()
            || paths.iter().any(|p| {
                p.as_str()
                    .is_none_or(|k| manifest["migrationPaths"].get(k).is_none())
            })
        {
            return Err("Startup probe references undeclared migration path".into());
        }
        if !caps
            .iter()
            .any(|v| v == "background" || v == "background.retain" || v == "host.background.retain")
        {
            return Err("Startup probe requires background capability".into());
        }
    }
    Ok(())
}
