use vg::mask::LogMasker;

#[test]
fn masks_single_secret() {
    let masker = LogMasker::new(&["sk_live_abc".to_string()]).unwrap();
    assert_eq!(masker.mask("key=sk_live_abc rest"), "key=***[MASKED]*** rest");
}

#[test]
fn masks_multiple_secrets() {
    let masker =
        LogMasker::new(&["sk_live_abc".to_string(), "db_password_xyz".to_string()]).unwrap();
    assert_eq!(
        masker.mask("sk_live_abc and db_password_xyz"),
        "***[MASKED]*** and ***[MASKED]***"
    );
}

#[test]
fn no_match_is_passthrough() {
    let masker = LogMasker::new(&["secret".to_string()]).unwrap();
    assert_eq!(masker.mask("harmless log line"), "harmless log line");
}

#[test]
fn empty_secrets_is_noop() {
    let masker = LogMasker::new(&[]).unwrap();
    assert_eq!(masker.mask("anything"), "anything");
}
