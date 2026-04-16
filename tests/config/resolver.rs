use serde_json::json;
use vg::config::resolver::{expand_template, resolve_value, SecretStores};

fn stores() -> SecretStores {
    SecretStores {
        global: json!({ "global": { "stripe": { "secret_key": "sk_live_abc" } } }),
        project: None,
    }
}

fn stores_with_project() -> SecretStores {
    SecretStores {
        global: json!({ "global": { "stripe": { "secret_key": "sk_live_abc" } } }),
        project: Some(json!({ "openai": { "api_key": "proj_key_xyz" } })),
    }
}

#[test]
fn resolve_plain_value() {
    let s = resolve_value("http://localhost:8080", &stores()).unwrap();
    assert_eq!(s, "http://localhost:8080");
}

#[test]
fn resolve_secret_ref() {
    let s = resolve_value("secret://global/stripe/secret_key", &stores()).unwrap();
    assert_eq!(s, "sk_live_abc");
}

#[test]
fn resolve_project_secret_ref() {
    let s = resolve_value("secret://project/openai/api_key", &stores_with_project()).unwrap();
    assert_eq!(s, "proj_key_xyz");
}

#[test]
fn resolve_project_secret_no_store() {
    // project scope without a project store loaded should be an error
    assert!(resolve_value("secret://project/openai/api_key", &stores()).is_err());
}

#[test]
fn expand_bearer_template() {
    let s = expand_template("Bearer ${secret://global/stripe/secret_key}", &stores()).unwrap();
    assert_eq!(s, "Bearer sk_live_abc");
}

#[test]
fn expand_project_bearer_template() {
    let s = expand_template(
        "Bearer ${secret://project/openai/api_key}",
        &stores_with_project(),
    )
    .unwrap();
    assert_eq!(s, "Bearer proj_key_xyz");
}

#[test]
fn expand_no_placeholder() {
    let s = expand_template("plain text", &stores()).unwrap();
    assert_eq!(s, "plain text");
}

#[test]
fn expand_unclosed_brace() {
    assert!(expand_template("${unclosed", &stores()).is_err());
}
