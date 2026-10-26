pub(super) fn installed_command(ctx: &CoreContext, id: &str, command: &str) -> Option<Installed> {
    let native = if id.starts_with("raycast.") {
        let slug = slug(id).ok()?;
        let canonical = super::v2_runtime::canonical_id(&slug);
        installed(ctx, &canonical).ok()
    } else if id.starts_with("com.tezbar.native.") {
        installed(ctx, id).ok()
    } else {
        None
    };
    if let Some(extension) = native {
        let active = extension.manifest["healthError"].is_null()
            && extension.manifest["commands"]
                .as_array()?
                .iter()
                .any(|c| c["name"] == command && c["runtime"] == "rust" && compatible(c, true));
        if active {
            return Some(extension);
        }
        if id.starts_with("com.tezbar.native.") {
            return None;
        }
    }
    if id.starts_with("com.tezbar.native.") {
        return None;
    }
    let extension = installed(ctx, id).ok()?;
    (compatible(&extension.manifest, false)
        && extension.manifest["commands"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|cmd| cmd["name"].as_str() == Some(command) && compatible(cmd, true)))
    .then_some(extension)
}
