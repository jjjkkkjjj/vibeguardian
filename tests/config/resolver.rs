use serde_json::json;
use vs::config::resolver::{expand_template, resolve_value};

fn store() -> serde_json::Value {
    json!({ "global": { "stripe": { "secret_key": "sk_live_abc" } } })
}

#[test]
fn resolve_plain_value() {
    let s = resolve_value("http://localhost:8080", &store()).unwrap();
    assert_eq!(s, "http://localhost:8080");
}

#[test]
fn resolve_secret_ref() {
    let s = resolve_value("secret://global/stripe/secret_key", &store()).unwrap();
    assert_eq!(s, "sk_live_abc");
}

#[test]
fn expand_bearer_template() {
    let s = expand_template("Bearer ${secret://global/stripe/secret_key}", &store()).unwrap();
    assert_eq!(s, "Bearer sk_live_abc");
}

#[test]
fn expand_no_placeholder() {
    let s = expand_template("plain text", &store()).unwrap();
    assert_eq!(s, "plain text");
}

#[test]
fn expand_unclosed_brace() {
    assert!(expand_template("${unclosed", &store()).is_err());
}
