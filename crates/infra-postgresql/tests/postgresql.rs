use launchlightly_infra_postgresql::{SeedOutcome, SuperAdminSeed, migrate, seed_super_admin};
use sqlx::PgPool;

#[sqlx::test]
#[ignore = "requires PostgreSQL"]
async fn embedded_migrations_can_run_twice(pool: PgPool) {
    migrate(&pool).await.expect("first migration run");
    migrate(&pool).await.expect("second migration run");

    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' \
         AND table_name IN ('users', 'accounts', 'sessions', 'verifications')",
    )
    .fetch_one(&pool)
    .await
    .expect("count Better Auth tables");

    assert_eq!(tables, 4);

    let canonical_email_schema: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_constraint \
         WHERE conname = 'users_email_is_canonical' \
         AND conrelid = 'users'::regclass",
    )
    .fetch_one(&pool)
    .await
    .expect("find canonical email constraint");
    let canonical_email_index: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pg_indexes \
         WHERE schemaname = 'public' \
         AND tablename = 'users' \
         AND indexname = 'users_email_canonical_unique'",
    )
    .fetch_one(&pool)
    .await
    .expect("find canonical email index");

    assert_eq!(canonical_email_schema, 1);
    assert_eq!(canonical_email_index, 1);
}

#[sqlx::test]
#[ignore = "requires PostgreSQL"]
async fn super_admin_seed_creates_one_authenticatable_identity(pool: PgPool) {
    migrate(&pool).await.expect("migrate");
    let seed = SuperAdminSeed::new("owner@example.com", "correct-horse-battery-staple", None)
        .expect("valid seed");

    let first = seed_super_admin(&pool, &seed).await.expect("first seed");
    let first_hash: String =
        sqlx::query_scalar("SELECT metadata->>'password_hash' FROM users WHERE email = $1")
            .bind("owner@example.com")
            .fetch_one(&pool)
            .await
            .expect("first password hash");

    let changed_password = SuperAdminSeed::new(
        "owner@example.com",
        "this-password-must-not-replace-the-first",
        Some("Changed Name".to_owned()),
    )
    .expect("valid second seed");
    let second = seed_super_admin(&pool, &changed_password)
        .await
        .expect("second seed");
    let different_email = SuperAdminSeed::new(
        "other-owner@example.com",
        "another-correct-horse-battery-staple",
        None,
    )
    .expect("valid different-email seed");
    let third = seed_super_admin(&pool, &different_email).await;

    assert!(matches!(first, SeedOutcome::Created { .. }));
    assert!(matches!(second, SeedOutcome::AlreadyExists { .. }));
    assert!(third.is_err());

    let user_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email = $1 AND role = 'super_admin' AND email_verified",
    )
    .bind("owner@example.com")
    .fetch_one(&pool)
    .await
    .expect("count seeded users");
    let account_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts a JOIN users u ON u.id = a.user_id \
         WHERE u.email = $1 AND a.provider_id = 'credential'",
    )
    .bind("owner@example.com")
    .fetch_one(&pool)
    .await
    .expect("count credential accounts");
    let super_admin_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'super_admin'")
            .fetch_one(&pool)
            .await
            .expect("count all super admins");
    let name: String = sqlx::query_scalar("SELECT name FROM users WHERE email = $1")
        .bind("owner@example.com")
        .fetch_one(&pool)
        .await
        .expect("seeded name");
    let (metadata_hash, account_hash): (String, String) = sqlx::query_as(
        "SELECT u.metadata->>'password_hash', a.password \
         FROM users u JOIN accounts a ON a.user_id = u.id \
         WHERE u.email = $1 AND a.provider_id = 'credential'",
    )
    .bind("owner@example.com")
    .fetch_one(&pool)
    .await
    .expect("stored credential hashes");

    assert_eq!(user_count, 1);
    assert_eq!(account_count, 1);
    assert_eq!(super_admin_count, 1);
    assert_eq!(name, "Super Admin");
    assert_eq!(metadata_hash, account_hash);
    assert_eq!(metadata_hash, first_hash);
}
