fn fallback<'a>(value: &'a Value, key: &str, default: &'a str) -> &'a str {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or(default)
}
