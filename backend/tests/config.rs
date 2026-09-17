use anyhow::Result;
use tap_backend::app::Config;

fn config(overrides: &[(&str, &str)]) -> Result<Config> {
    Config::from_lookup(|key| {
        overrides
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).into())
    })
}

#[test]
fn defaults_work_with_security_disabled_opensearch() {
    let config = config(&[]).unwrap();
    assert_eq!(config.address.to_string(), "0.0.0.0:3000");
    assert!(config.opensearch_credentials.is_none());
    assert_eq!(
        config.database_url,
        "postgresql://tap:tap@localhost:5432/tap"
    );
    assert_eq!(config.database_max_connections, 10);
    assert_eq!(config.dependency_timeout.as_millis(), 3000);
    assert_eq!(config.log_filter, "tap_backend=debug,tower_http=debug");
}

#[test]
fn rejects_invalid_configuration() {
    for overrides in [
        vec![("PORT", "0")],
        vec![("PORT", "65536")],
        vec![("DATABASE_URL", "")],
        vec![("DATABASE_URL", "invalid")],
        vec![("REDIS_URL", "http://localhost")],
        vec![("OPENSEARCH_NODE", "ftp://localhost")],
        vec![("OPENSEARCH_USERNAME", "admin")],
        vec![("CORS_ORIGIN", "http://localhost:4200/path")],
        vec![("APP_ENV", "unknown")],
        vec![("DATABASE_MAX_CONNECTIONS", "0")],
        vec![("DEPENDENCY_TIMEOUT_MS", "0")],
        vec![("RUST_LOG", "tap_backend=invalid")],
    ] {
        assert!(config(&overrides).is_err(), "{overrides:?}");
    }
}

#[test]
fn test_profile_has_separate_database_and_redis_defaults() {
    let config = config(&[("APP_ENV", "test")]).unwrap();
    assert_eq!(config.address.to_string(), "127.0.0.1:3001");
    assert_eq!(
        config.database_url,
        "postgresql://tap:tap@localhost:5432/tap_test"
    );
    assert_eq!(config.database_max_connections, 2);
    assert_eq!(config.redis_url, "redis://localhost:6379/1");
}

#[test]
fn production_requires_database_credentials_and_uses_service_names() {
    let error = config(&[("APP_ENV", "prod")]).err().unwrap();
    assert!(error.to_string().contains("DATABASE_URL"));
    let config = config(&[
        ("APP_ENV", "prod"),
        (
            "DATABASE_URL",
            "postgresql://deploy:secret@postgres:5432/tap",
        ),
    ])
    .unwrap();
    assert_eq!(config.redis_url, "redis://redis:6379");
    assert_eq!(config.opensearch_url.as_str(), "http://opensearch:9200/");
    assert_eq!(config.cors_origin, "http://localhost:4000");
    assert_eq!(config.log_filter, "tap_backend=info,tower_http=info");
}

#[test]
fn environment_overrides_yaml_defaults() {
    let config = config(&[
        ("APP_ENV", "test"),
        ("HOST", "0.0.0.0"),
        ("PORT", "8080"),
        ("DATABASE_URL", "postgresql://tap:tap@postgres:5432/custom"),
        ("DATABASE_MAX_CONNECTIONS", "20"),
        ("REDIS_URL", "redis://redis:6379/2"),
        ("OPENSEARCH_NODE", "https://search.example.com"),
        ("OPENSEARCH_USERNAME", "admin"),
        ("OPENSEARCH_PASSWORD", "secret"),
        ("CORS_ORIGIN", "https://app.example.com"),
        ("DEPENDENCY_TIMEOUT_MS", "1500"),
        ("RUST_LOG", "tap_backend=trace"),
    ])
    .unwrap();
    assert_eq!(config.address.to_string(), "0.0.0.0:8080");
    assert_eq!(
        config.database_url,
        "postgresql://tap:tap@postgres:5432/custom"
    );
    assert_eq!(config.database_max_connections, 20);
    assert_eq!(config.redis_url, "redis://redis:6379/2");
    assert_eq!(
        config.opensearch_url.as_str(),
        "https://search.example.com/"
    );
    assert_eq!(
        config.opensearch_credentials,
        Some(("admin".into(), "secret".into()))
    );
    assert_eq!(config.cors_origin, "https://app.example.com");
    assert_eq!(config.dependency_timeout.as_millis(), 1500);
    assert_eq!(config.log_filter, "tap_backend=trace");
}

#[test]
fn custom_configuration_directory_loads_selected_file() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config");
    let settings = config(&[
        ("APP_ENV", "test"),
        ("CONFIG_DIR", directory.to_str().unwrap()),
    ])
    .unwrap();
    assert_eq!(settings.database_max_connections, 2);
    let missing = directory.join("missing-directory");
    assert!(config(&[("CONFIG_DIR", missing.to_str().unwrap())]).is_err());
}

#[test]
fn rejects_malformed_yaml_and_unknown_settings() {
    assert!(Config::from_yaml("server: [", |_| None).is_err());
    let yaml = include_str!("../config/dev.yml").replace("max_connections:", "max_connection:");
    let error = Config::from_yaml(&yaml, |_| None).err().unwrap();
    assert!(format!("{error:#}").contains("unknown field"));
}
