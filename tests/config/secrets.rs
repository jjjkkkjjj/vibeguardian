use serde_json::json;
use vg::config::secrets::resolve;

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

#[test]
fn resolve_project_path() {
    // Project store does not have the "project/" prefix in paths —
    // the namespace is implicit in the file location.
    let store = json!({ "openai": { "api_key": "proj_key_abc" } });
    let val = resolve(&store, "openai/api_key").unwrap();
    assert_eq!(val, "proj_key_abc");
}
