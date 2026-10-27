//! Actual pinned functions with narrow dependency stubs; proposed patch runs separately.
use serde_json::{json, Value};
#[allow(dead_code, clippy::all)]
mod launcher {
    include!("../evidence/host/validate-launcher.rs");
}
#[allow(dead_code, clippy::all)]
mod process_install {
    include!("../evidence/host/validate-package.rs");
}
#[test]
fn actual_pinned_host_and_schema_validator_positive_negative() {
    let manifest: Value = serde_json::from_str(include_str!("../extension.json")).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../evidence/host/extension-v3.schema.json")).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&manifest));
    assert!(process_install::validate(&manifest).is_ok());
    for (field, v) in [
        ("schemaVersion", json!(2)),
        ("runtime", json!("javascript")),
        ("execution", json!("embedded")),
        ("protocolVersion", json!(2)),
        ("id", json!("raycast.ipapi-is")),
        ("capabilities", json!(["arbitrary-method"])),
    ] {
        let mut m = manifest.clone();
        m[field] = v;
        assert!(!validator.is_valid(&m), "{field}");
        assert!(process_install::validate(&m).is_err(), "{field}");
    }
    for (field, v) in [
        ("refreshMs", json!(499)),
        ("timeoutMs", json!(300001)),
        ("concurrency", json!("parallel")),
    ] {
        let mut m = manifest.clone();
        m["commands"][0][field] = v;
        assert!(!validator.is_valid(&m));
        assert!(process_install::validate(&m).is_err());
    }
    let mut m = manifest.clone();
    m["commands"]
        .as_array_mut()
        .unwrap()
        .push(manifest["commands"][0].clone());
    assert!(process_install::validate(&m).is_err());
    let mut m = manifest.clone();
    m["unexpected"] = json!(true);
    assert!(!validator.is_valid(&m));
    for path in ["../x", "/x", "bin/../x", "bin\\x", "bin/CON.exe", "bin/x."] {
        assert!(process_install::relative(path).is_err());
    }
}
mod fixture {
    use serde_json::{json, Value};
    use std::path::{Component, Path, PathBuf};
    #[derive(Clone)]
    pub struct Installed {
        pub id: String,
        pub slug: String,
        pub path: PathBuf,
        pub resource_dir: PathBuf,
        pub manifest: Value,
    }
    pub struct CoreContext {
        pub packages: Vec<Installed>,
    }
    pub fn collect(ctx: &CoreContext) -> Vec<Installed> {
        ctx.packages.clone()
    }
    pub fn installed(ctx: &CoreContext, id: &str) -> Result<Installed, String> {
        ctx.packages
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or("missing".into())
    }
    pub fn compatible(c: &Value, _native: bool) -> bool {
        c["platforms"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v == "macOS"))
    }
    pub mod v2_runtime {
        pub fn canonical_id(s: &str) -> String {
            format!("com.tezbar.native.{s}")
        }
    }
    include!("../evidence/host/slug-before.rs");
    include!("../evidence/host/fallback-before.rs");
    pub mod before {
        use super::*;
        include!("../evidence/host/installed_command-before.rs");
        include!("../evidence/host/icon_path-before.rs");
        include!("../evidence/host/command_icon_path-before.rs");
        pub fn icon(e: &Installed) -> Option<String> {
            icon_path(e)
        }
        pub fn command_icon(e: &Installed, c: &Value) -> Option<String> {
            command_icon_path(e, c)
        }
    }
    pub mod after {
        use super::*;
        include!("../evidence/host/installed_command-after.rs");
        include!("../evidence/host/icon_path-after.rs");
        include!("../evidence/host/command_icon_path-after.rs");
        pub fn icon(e: &Installed) -> Option<String> {
            icon_path(e)
        }
        pub fn command_icon(e: &Installed, c: &Value) -> Option<String> {
            command_icon_path(e, c)
        }
    }
    pub fn context(root: &Path) -> CoreContext {
        CoreContext {
            packages: vec![Installed {
                id: "com.tezbar.native.ipapi-is".into(),
                slug: "ipapi-is".into(),
                path: root.join("canonical"),
                resource_dir: root.join("canonical/versions/0.1.0"),
                manifest: serde_json::from_str(include_str!("../extension.json")).unwrap(),
            }],
        }
    }
    #[test]
    fn exact_own_alias_routes_only_verified_covered_rust_command() {
        let ctx = context(Path::new("unused"));
        assert!(
            before::installed_command(&ctx, "com.tezbar.ipapi-is", "lookup-ip-or-asn").is_none()
        );
        for id in [
            "com.tezbar.ipapi-is",
            "raycast.ipapi-is",
            "com.tezbar.native.ipapi-is",
        ] {
            assert!(after::installed_command(&ctx, id, "lookup-ip-or-asn").is_some());
            assert!(after::installed_command(&ctx, id, "uncovered").is_none());
        }
        assert!(
            after::installed_command(&ctx, "com.tezbar.ipapi-is-other", "lookup-ip-or-asn")
                .is_none()
        );
        let mut ctx = ctx;
        ctx.packages[0].manifest["healthError"] = json!("tampered");
        assert!(
            after::installed_command(&ctx, "com.tezbar.ipapi-is", "lookup-ip-or-asn").is_none()
        );
    }
    #[test]
    fn active_version_icons_resolve_without_moving_preference_or_uninstall_root() {
        let root = std::env::temp_dir().join(format!("host-icon-fixture-{}", std::process::id()));
        std::fs::create_dir_all(root.join("canonical/versions/0.1.0/resources")).unwrap();
        std::fs::write(
            root.join("canonical/versions/0.1.0/resources/ipapi-is.png"),
            include_bytes!("../resources/ipapi-is.png"),
        )
        .unwrap();
        let ctx = context(&root);
        let e = &ctx.packages[0];
        assert!(before::icon(e).is_none());
        assert!(before::command_icon(e, &e.manifest["commands"][0]).is_none());
        assert!(after::icon(e).unwrap().contains("versions/0.1.0/resources"));
        assert!(after::command_icon(e, &e.manifest["commands"][0]).is_some());
        assert_eq!(e.path, root.join("canonical"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
