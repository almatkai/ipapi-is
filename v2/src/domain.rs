//! Source-ordered metadata. Remote content never becomes Markdown or renderer links.
use serde_json::{json, Value};

pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_NODES: usize = 16_384;
pub const MAX_ROWS: usize = 1024;
pub fn truth(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|n| n != 0.0),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}
pub fn js(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => ryu_js::Buffer::new()
            .format_finite(n.as_f64().unwrap())
            .to_owned(),
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => a
            .iter()
            .map(|v| if v.is_null() { String::new() } else { js(v) })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}
// JSON.parse/JSON.stringify use IEEE754 numbers and integer-index property ordering.
pub fn stringify(v: &Value) -> String {
    match v {
        Value::Number(_) => js(v),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(stringify).collect::<Vec<_>>().join(",")
        ),
        Value::Object(o) => {
            let mut keys = o.keys().collect::<Vec<_>>();
            fn index(s: &str) -> Option<u32> {
                let n = s.parse::<u32>().ok()?;
                (n != u32::MAX && n.to_string() == s).then_some(n)
            }
            keys.sort_by_key(|k| index(k).map_or((1, 0), |n| (0, n)));
            format!(
                "{{{}}}",
                keys.iter()
                    .map(|k| format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap(),
                        stringify(&o[*k])
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => serde_json::to_string(v).unwrap(),
    }
}
pub fn decode(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Response exceeds the size limit".into());
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|_| "Invalid JSON response")?;
    let mut pending = vec![(&v, 0)];
    let mut nodes = 0;
    while let Some((v, depth)) = pending.pop() {
        nodes += 1;
        if nodes > MAX_NODES || depth > 24 {
            return Err("Response exceeds the structural limit".into());
        }
        match v {
            Value::Object(o) => pending.extend(o.values().map(|v| (v, depth + 1))),
            Value::Array(a) => {
                if a.len() > MAX_ROWS {
                    return Err("Response exceeds the row limit".into());
                }
                pending.extend(a.iter().map(|v| (v, depth + 1)));
            }
            Value::String(s) if s.len() > 16384 => {
                return Err("Response field exceeds the size limit".into())
            }
            _ => {}
        }
    }
    if !v.is_object() || !v["elapsed_ms"].is_number() {
        return Err("Incomplete response".into());
    }
    if v.get("error").is_some() {
        if !v["error"].is_string() {
            return Err("Incomplete error response".into());
        }
    } else if v.get("ip").is_some() {
        if !v["ip"].is_string() {
            return Err("Incomplete IP response".into());
        }
        if truth(&v["company"]) && !v["company"]["domain"].is_string() {
            return Err("Incomplete company response".into());
        }
        if truth(&v["asn"]) {
            validate_asn(&v["asn"])?;
        }
    } else {
        validate_asn(&v)?;
    }
    // Every declared structured field must have its source-compatible type.
    for key in ["datacenter", "company", "abuse", "location", "vpn"] {
        if v.get("ip").is_some() && truth(&v[key]) && !v[key].is_object() {
            return Err("Invalid response section".into());
        }
    }
    for (o, fields) in [
        (&v, &["prefixes", "prefixesIPv6"][..]),
        (&v["location"], &["other"][..]),
    ] {
        for key in fields {
            if truth(&o[key])
                && !o[*key]
                    .as_array()
                    .is_some_and(|a| a.iter().all(Value::is_string))
            {
                return Err("Invalid response tags".into());
            }
        }
    }
    Ok(v)
}
fn validate_asn(v: &Value) -> Result<(), String> {
    if !v["asn"].is_number() || !v["country"].is_string() {
        return Err("Incomplete ASN response".into());
    }
    if truth(&v["abuse"])
        && !v["abuse"].is_string()
        && !v["abuse"]
            .as_array()
            .is_some_and(|a| a.iter().all(Value::is_string))
    {
        return Err("Invalid ASN abuse response".into());
    }
    Ok(())
}
pub fn query(input: &str) -> Result<url::Url, String> {
    // V1 does not parse, trim, normalize, or resolve input: the API owns validation.
    if input.len() > 1024 || input.chars().any(char::is_control) {
        return Err("Input exceeds the limit or contains control characters".into());
    }
    let mut u = url::Url::parse("https://api.ipapi.is").unwrap();
    u.query_pairs_mut().append_pair("q", input);
    Ok(u)
}
pub fn safe_link(raw: &str) -> Option<String> {
    if raw.len() > 2048
        || raw.trim() != raw
        || raw
            .chars()
            .any(|c| c.is_control() || c == '\\' || c.is_whitespace())
    {
        return None;
    }
    if !raw.starts_with("https://") && !raw.starts_with("http://") {
        return None;
    }
    let u = url::Url::parse(raw).ok()?;
    if !matches!(u.scheme(), "https" | "http")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return None;
    }
    // No query/fragment auth reflection (the source has no legitimate key preference).
    if u.query_pairs().any(|(k, _)| {
        matches!(
            k.to_ascii_lowercase().as_str(),
            "key" | "token" | "api_key" | "apikey" | "authorization" | "access_token"
        )
    }) || u.fragment().is_some()
    {
        return None;
    }
    Some(raw.into())
}
#[derive(Default)]
pub struct Metadata {
    pub rows: Vec<Value>,
    pub links: Vec<(String, String)>,
}
impl Metadata {
    fn label(&mut self, title: &str, value: Option<&Value>) {
        let mut p = json!({"title":title});
        if let Some(v) = value {
            p["text"] = v.clone();
        }
        self.rows
            .push(json!({"type":"Detail.Metadata.Label","props":p}));
    }
    fn text(&mut self, title: &str, text: &str) {
        self.label(title, Some(&json!(text)));
    }
    fn fields(&mut self, d: &Value, fields: &[(&str, &str)]) {
        for (t, k) in fields {
            self.label(t, d.get(*k));
        }
    }
    fn boolean(&mut self, title: &str, v: &Value) {
        self.rows.push(json!({"type":"Detail.Metadata.Label","props":{"title":title,"text":if truth(v){"Yes"}else{"No"},"icon":if truth(v){"Check"}else{"Xmark"}}}));
    }
    fn section(&mut self, title: &str) {
        self.rows
            .push(json!({"type":"Detail.Metadata.Separator","props":{}}));
        self.text(title, "...");
    }
    fn tags(&mut self, title: &str, a: &Value) {
        self.rows.push(json!({"type":"Detail.Metadata.TagList","props":{"title":title},"children":a.as_array().unwrap().iter().map(|v| json!({"type":"Detail.Metadata.TagList.Item","props":{"text":v}})).collect::<Vec<_>>()}));
    }
    fn link(&mut self, title: &str, value: Option<&Value>, target: Option<String>) {
        self.label(title, value);
        if let Some(url) = target.and_then(|s| safe_link(&s)) {
            self.links.push((format!("Open {title}"), url));
        }
    }
    fn asn(&mut self, d: &Value) {
        self.text("ASN", &js(&d["asn"]));
        self.fields(
            d,
            &[("Abuser Score", "abuser_score"), ("Description", "descr")],
        );
        // Pinned renderer's metadata icons do not fetch remote flag images; honest fallback.
        self.rows.push(json!({"type":"Detail.Metadata.Label","props":{"title":"Country","text":d["country"],"icon":"Map"}}));
        self.boolean("Active", &d["active"]);
        if truth(&d["org"]) {
            self.label("Organization", d.get("org"));
        }
        if truth(&d["abuse"]) {
            if d["abuse"].is_array() {
                self.tags("Abuse", &d["abuse"]);
            } else {
                self.link("Abuse", d.get("abuse"), None);
            }
        }
        if truth(&d["domain"]) {
            self.link(
                "Domain",
                d.get("domain"),
                d["domain"].as_str().map(|s| format!("https://{s}")),
            );
        }
        for (t, k) in [
            ("Type", "type"),
            ("Created", "created"),
            ("Updated", "updated"),
            ("Regional Internet Registry", "rir"),
        ] {
            if truth(&d[k]) {
                self.label(t, d.get(k));
            }
        }
        self.label("WHOIS", d.get("whois"));
        if truth(&d["route"]) {
            self.label("Route", d.get("route"));
        }
    }
}
pub fn metadata(d: &Value) -> Metadata {
    let mut m = Metadata::default();
    m.text("Time Elapsed (ms)", &js(&d["elapsed_ms"]));
    if d.get("error").is_some() {
        // Remote errors may reflect credentials or URLs. Never echo raw server error text.
        m.text(
            "Error",
            "The service reported an error. Revalidate to retry.",
        );
        return m;
    }
    if d.get("ip").is_none() {
        m.asn(d);
        for (t, k) in [
            ("Prefixes", "prefixes"),
            ("Prefixes (IPv6)", "prefixesIPv6"),
        ] {
            if truth(&d[k]) {
                m.tags(t, &d[k]);
            }
        }
        return m;
    }
    m.fields(d, &[("IP", "ip"), ("Regional Internet Registry", "rir")]);
    for (t, k) in [
        ("Bogon (Non Routable)", "is_bogon"),
        ("Mobile ISP", "is_mobile"),
        ("Crawler", "is_crawler"),
        ("Datacenter", "is_datacenter"),
        ("TOR Exit Node", "is_tor"),
        ("Proxy Exit Node", "is_proxy"),
        ("VPN Exit Node", "is_vpn"),
        ("Abuser", "is_abuser"),
    ] {
        m.boolean(t, &d[k]);
    }
    m.section("DATACENTER");
    if truth(&d["datacenter"]) {
        m.fields(
            &d["datacenter"],
            &[
                ("Datacenter", "datacenter"),
                ("Network", "network"),
                ("Domain", "domain"),
                ("Region", "region"),
                ("Service", "service"),
                ("Network Border Group", "network_border_group"),
                ("Code", "code"),
                ("City", "city"),
                ("State", "state"),
                ("Name", "name"),
                ("Country", "country"),
            ],
        );
    }
    m.section("COMPANY");
    let c = &d["company"];
    if truth(c) {
        m.label("Name", c.get("name"));
        let domain = c["domain"].as_str().unwrap();
        m.text(
            "Domain",
            &if domain.contains("http") {
                domain.into()
            } else {
                format!("https://{domain}")
            },
        );
        m.fields(c, &[("Type", "type"), ("Network", "network")]);
        m.link(
            "WHOIS",
            c.get("whois"),
            c["whois"].as_str().map(str::to_owned),
        );
    }
    m.section("ABUSE");
    let a = &d["abuse"];
    if truth(a) {
        m.fields(
            a,
            &[
                ("Name", "name"),
                ("Address", "address"),
                ("Country", "country"),
            ],
        );
        m.link("Email", a.get("email"), None);
        m.link("Phone", a.get("phone"), None);
    }
    m.section("ASN");
    if truth(&d["asn"]) {
        m.asn(&d["asn"]);
    }
    m.section("LOCATION");
    let l = &d["location"];
    if truth(l) {
        m.boolean("EU Member", &l["is_eu_member"]);
        m.fields(
            l,
            &[
                ("Continent", "continent"),
                ("Country", "country"),
                ("Country Code", "country_code"),
                ("State", "state"),
                ("City", "city"),
            ],
        );
        for (t, k) in [("Latitude", "latitude"), ("Longitude", "longitude")] {
            m.text(
                t,
                &if truth(&l[k]) {
                    js(&l[k])
                } else {
                    "N/A".into()
                },
            );
        }
        m.fields(
            l,
            &[
                ("ZIP", "zip"),
                ("Timezone", "timezone"),
                ("Local Time", "local_time"),
            ],
        );
        m.text(
            "Local Time (UNIX)",
            &if truth(&l["local_time_unix"]) {
                js(&l["local_time_unix"])
            } else {
                "N/A".into()
            },
        );
        m.boolean("Daylight Saving Time", &l["is_dst"]);
        if l["other"].as_array().is_some_and(|a| !a.is_empty()) {
            m.tags("Other", &l["other"]);
        }
    }
    m.section("VPN");
    let v = &d["vpn"];
    if truth(v) {
        m.label("Region", v.get("region"));
        m.text(
            "Last Seen",
            &if truth(&v["last_seen"]) {
                js(&v["last_seen"])
            } else {
                "N/A".into()
            },
        );
        m.fields(v, &[("Type", "type"), ("Service", "service")]);
        m.link("URL", v.get("url"), v["url"].as_str().map(str::to_owned));
    }
    m
}
pub fn heading(input: &str) -> String {
    // Numeric character references make input inert in Markdown, including brackets/HTML.
    let safe: String = input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " .:-_".contains(c) {
                c.to_string()
            } else {
                format!("&#{};", c as u32)
            }
        })
        .collect();
    format!(
        "# {}: {}\n---\n",
        if input.to_uppercase().contains("AS") {
            "ASN"
        } else {
            "IP"
        },
        if input.is_empty() { "Your IP" } else { &safe }
    )
}
