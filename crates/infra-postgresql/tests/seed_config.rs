use launchlightly_infra_postgresql::SuperAdminSeed;

#[test]
fn super_admin_seed_rejects_missing_credentials() {
    assert!(SuperAdminSeed::new("", "valid-password", None).is_err());
    assert!(SuperAdminSeed::new("owner@example.com", "", None).is_err());
}

#[test]
fn super_admin_seed_uses_the_authentication_credential_policy() {
    assert!(SuperAdminSeed::new("not-an-email", "valid-password", None).is_err());
    assert!(SuperAdminSeed::new("owner@example.com", "short", None).is_err());
    assert!(SuperAdminSeed::new("owner@example.com", "x".repeat(129), None).is_err());
    assert!(SuperAdminSeed::new("owner@example.com", "12345678", None).is_ok());
}
