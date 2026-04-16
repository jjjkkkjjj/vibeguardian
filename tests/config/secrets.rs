use serde_json::json;
use vs::config::secrets::resolve;

#[test]
fn resolve_nested_path() {
    let store = json!({ "global": { "stripe": { "secret_key": "sk_test_123" } } });
    let val = resolve(&store, "global/stripe/secret_key").unwrap();
    assert_eq!(val, "sk_test_123");
}

#[test]
fn resolve_missing_path() {
    let store = json!({});
    assert!(resolve(&store, "global/missing").is_err());
}
