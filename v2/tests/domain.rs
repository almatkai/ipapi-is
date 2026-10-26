use serde_json::{json, Value};
use tezbar_ipapi_is::{acknowledgement, domain::*};
#[test]
fn finite_original_oracle() {
    let fixtures: Value = serde_json::from_str(include_str!("../fixtures/oracle.json")).unwrap();
    for c in fixtures.as_array().unwrap() {
        let mut data = decode(c["wire"].as_str().unwrap().as_bytes()).unwrap();
        if data.get("error").is_some() {
            data["error"] = json!("The service reported an error. Revalidate to retry.");
        }
        let m = metadata(&data);
        for (i, (a, b)) in m
            .rows
            .iter()
            .zip(c["metadata"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(a, b, "{} row {i}", c["name"]);
        }
        assert_eq!(m.rows.len(), c["metadata"].as_array().unwrap().len());
        assert_eq!(
            stringify(&data),
            c["copy"].as_str().unwrap(),
            "{}",
            c["name"]
        );
        assert_eq!(
            heading(c["input"].as_str().unwrap()),
            c["heading"],
            "{}",
            c["name"]
        );
    }
}
#[test]
fn bounds_and_partial_contracts() {
    for bytes in [
        b"{}".as_slice(),
        b"null",
        b"[]",
        b"{\"elapsed_ms\":0,\"asn\":1}",
        b"{\"elapsed_ms\":0,\"ip\":\"x\",\"company\":{}}",
        b"{\"elapsed_ms\":0,\"ip\":\"x\",\"location\":{\"other\":42}}",
    ] {
        assert!(decode(bytes).is_err());
    }
    assert!(decode(&vec![b' '; MAX_BYTES + 1]).is_err());
    let mut v = json!({"ip":"192.0.2.1","elapsed_ms":0});
    v["extra"] = json!(vec!["x"; MAX_ROWS + 1]);
    assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
    let mut deep = json!(0);
    for _ in 0..30 {
        deep = json!({"a":deep});
    }
    v["extra"] = deep;
    assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
    v["extra"] = json!(vec![vec![0; 100]; 200]);
    assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn query_is_opaque_and_encoded_no_dns_or_asn_normalization() {
    for q in [
        "",
        "192.0.2.1",
        "2001:db8::1",
        "AS0",
        "as64512",
        "64512",
        "domain.invalid",
        " 192.0.2.1 ",
        "x&key=fake#part",
        "[::1]",
        "ß",
    ] {
        let u = query(q).unwrap();
        assert_eq!(u.host_str(), Some("api.ipapi.is"));
        assert_eq!(u.scheme(), "https");
        assert_eq!(
            u.query_pairs().collect::<Vec<_>>(),
            vec![("q".into(), q.into())]
        );
        assert!(u.fragment().is_none());
    }
    assert!(query("a\nb").is_err());
    assert!(query(&"a".repeat(1025)).is_err());
}
#[test]
fn no_untrusted_effect_destinations() {
    for s in [
        "javascript:alert(1)",
        "file:///tmp/x",
        "mailto:abuse@example.invalid",
        "tel:123",
        "https://u:p@example.invalid/",
        "https://example.invalid/?key=fake",
        "https://example.invalid/#token=fake",
        "https://example.invalid\\@evil.invalid/",
        " https://example.invalid/",
        "https://example.invalid/a b",
    ] {
        assert!(safe_link(s).is_none(), "{s}");
    }
    assert_eq!(
        safe_link("https://example.invalid/path?q=hello%20world"),
        Some("https://example.invalid/path?q=hello%20world".into())
    );
    assert!(!heading("[x](https://evil.invalid) ![y](z) <img>").contains("[x]"));
}
#[test]
fn acknowledgements_are_method_specific_closed_objects() {
    assert!(acknowledgement(&json!({"ok":true})));
    assert!(acknowledgement(&json!({"ok":true,"message":"Copied"})));
    for v in [
        Value::Null,
        json!(true),
        json!({}),
        json!({"ok":false}),
        json!({"ok":true,"error":"contradiction"}),
        json!({"ok":true,"message":4}),
        json!({"ok":true,"extra":1}),
    ] {
        assert!(!acknowledgement(&v));
    }
}
