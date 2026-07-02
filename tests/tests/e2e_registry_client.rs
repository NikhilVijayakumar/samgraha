use std::fs;
use std::path::{Path, PathBuf};

use common::config::{DependencyConfig, ResolverConfig, SamgrahaConfig};
use schemas::manifest::{
    AuditSummary, CachedRepoMetadata, CompilerInfo, KnowledgeLocation, RepoIdentity,
    RepositoryManifest, RepositoryStatus,
};
use registry::registry_db::RegistryDb;
use services::registry_client::{FileRegistryClient, RegistryClient};
use services::resolution::KnowledgeResolver;
use uuid::Uuid;

fn create_test_env(name: &str) -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join("samgraha_test").join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root.join(".samgraha")).unwrap();
    fs::create_dir_all(&root.join("repos")).unwrap();
    let repos = root.join("repos");
    (root, repos)
}

fn make_manifest(
    id: &str,
    uuid: Uuid,
    root: &Path,
    revision: u64,
    exports: Vec<String>,
    status: &str,
) -> RepositoryManifest {
    RepositoryManifest {
        repository: RepoIdentity {
            id: id.to_string(),
            name: id.to_string(),
            uuid,
        },
        revision,
        compiler: CompilerInfo {
            version: "0.1.0".into(),
        },
        audit: AuditSummary {
            status: status.to_string(),
            last_audit: None,
        },
        repository_root: root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation {
            location: root.join(".samgraha").join("knowledge.db").to_string_lossy().to_string(),
        },
        exports,
        capabilities: vec![],
        dependencies: vec![],
        generated_at: "2026-06-27T12:00:00Z".into(),
    }
}

fn write_manifest(root: &Path, manifest: &RepositoryManifest) {
    let json = serde_json::to_string_pretty(manifest).unwrap();
    fs::write(root.join(".samgraha").join("manifest.json"), &json).unwrap();
}

fn init_repo(repos: &Path, name: &str) -> PathBuf {
    let root = repos.join(name);
    fs::create_dir_all(&root.join(".samgraha")).unwrap();
    root
}

// ── FileRegistryClient lifecycle ──────────────────────────────────────────

#[test]
fn test_register_and_list() {
    let (root, repos) = create_test_env("register_list");
    let repo_root = init_repo(&repos, "test-repo");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("test-repo", uuid, &repo_root, 1, vec!["arch".into()], "PASS");

    client.register(&manifest).unwrap();

    let entries = client.list().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].repository.id, "test-repo");
    assert_eq!(entries[0].repository.uuid, uuid);
    assert_eq!(entries[0].revision, 1);
    assert_eq!(entries[0].exports, vec!["arch"]);
}

#[test]
fn test_register_overwrite_same_id() {
    let (root, repos) = create_test_env("register_overwrite");
    let repo_root = init_repo(&repos, "same-id-repo");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let m1 = make_manifest("same-id", uuid, &repo_root, 1, vec!["arch".into()], "PASS");
    let m2 = make_manifest("same-id", uuid, &repo_root, 2, vec!["feature".into()], "PASS");
    client.register(&m1).unwrap();
    client.register(&m2).unwrap();

    let entries = client.list().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].repository.uuid, uuid);
    assert_eq!(entries[0].revision, 2);
}

#[test]
fn test_register_uuid_mismatch_rejected() {
    let (root, repos) = create_test_env("register_uuid_mismatch");
    let repo_root = init_repo(&repos, "uuid-mismatch");
    let client = FileRegistryClient::new(&root);
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let m1 = make_manifest("same-id", uuid1, &repo_root, 1, vec![], "PASS");
    let m2 = make_manifest("same-id", uuid2, &repo_root, 2, vec![], "PASS");
    client.register(&m1).unwrap();
    let err = client.register(&m2).unwrap_err();
    assert!(err.to_string().contains("UUID mismatch"));
}

#[test]
fn test_unregister_by_uuid() {
    let (root, repos) = create_test_env("unregister");
    let repo_root = init_repo(&repos, "to-remove");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("to-remove", uuid, &repo_root, 1, vec![], "PASS");
    client.register(&manifest).unwrap();
    assert_eq!(client.list().unwrap().len(), 1);

    client.unregister(&uuid).unwrap();
    assert_eq!(client.list().unwrap().len(), 0);
}

#[test]
fn test_unregister_unknown_uuid_fails() {
    let (root, repos) = create_test_env("unregister_unknown");
    let repo_root = init_repo(&repos, "keep");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("keep", uuid, &repo_root, 1, vec![], "PASS");
    client.register(&manifest).unwrap();

    let bogus = Uuid::new_v4();
    let err = client.unregister(&bogus).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_get_metadata() {
    let (root, repos) = create_test_env("get_metadata");
    let repo_root = init_repo(&repos, "get-me");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("get-me", uuid, &repo_root, 5, vec!["arch".into(), "sec".into()], "PASS");
    client.register(&manifest).unwrap();

    let meta = client.get_metadata(&uuid).unwrap().unwrap();
    assert_eq!(meta.repository.id, "get-me");
    assert_eq!(meta.revision, 5);
    assert!(meta.exports.contains(&"arch".to_string()));
}

#[test]
fn test_discover_by_uuid() {
    let (root, repos) = create_test_env("discover_uuid");
    let repo_root = init_repo(&repos, "disc-uuid");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("disc-uuid", uuid, &repo_root, 1, vec![], "PASS");
    client.register(&manifest).unwrap();

    let results = client.discover(&services::registry_client::RegistryQuery {
        uuid: Some(uuid),
        ..Default::default()
    }).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_discover_by_id() {
    let (root, repos) = create_test_env("discover_id");
    let repo_root = init_repo(&repos, "disc-id");
    let client = FileRegistryClient::new(&root);
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("disc-id", uuid, &repo_root, 1, vec![], "PASS");
    client.register(&manifest).unwrap();

    let results = client.discover(&services::registry_client::RegistryQuery {
        id: Some("disc-id".into()),
        ..Default::default()
    }).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_discover_by_export() {
    let (root, repos) = create_test_env("discover_export");
    let repo_a = init_repo(&repos, "has-arch");
    let repo_b = init_repo(&repos, "has-feat");
    let client = FileRegistryClient::new(&root);
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let m1 = make_manifest("has-arch", uuid1, &repo_a, 1, vec!["arch".into()], "PASS");
    let m2 = make_manifest("has-feat", uuid2, &repo_b, 1, vec!["feature".into()], "PASS");
    client.register(&m1).unwrap();
    client.register(&m2).unwrap();

    let results = client.discover(&services::registry_client::RegistryQuery {
        export: Some("arch".into()),
        ..Default::default()
    }).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].repository.id, "has-arch");
}

// ── TTL / expiry ─────────────────────────────────────────────────────────

#[test]
fn test_expired_metadata() {
    let (_root, repos) = create_test_env("expired");
    let repo_root = init_repo(&repos, "exp");
    let mut meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "exp".into(),
            name: "exp".into(),
            uuid: Uuid::new_v4(),
        },
        revision: 1,
        repository_root: repo_root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation {
            location: "dummy".into(),
        },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2020-01-01T00:00:00Z".into(),
        expires: "2020-01-02T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert!(meta.is_expired());

    meta.expires = "2099-01-01T00:00:00Z".into();
    assert!(!meta.is_expired());
}

#[test]
fn test_status_registered() {
    let (root, _repos) = create_test_env("status_ok");
    let now = std::time::SystemTime::now();
    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "ok".into(),
            name: "ok".into(),
            uuid: Uuid::new_v4(),
        },
        revision: 1,
        repository_root: root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2026-06-27T12:00:00Z".into(),
        expires: "2099-01-01T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(now), RepositoryStatus::Registered);
}

#[test]
fn test_status_missing() {
    let (_root, _repos) = create_test_env("status_missing");
    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "missing".into(),
            name: "missing".into(),
            uuid: Uuid::new_v4(),
        },
        revision: 1,
        repository_root: Path::new("Z:\\nonexistent\\path\\that\\cant\\exist").to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2026-06-27T12:00:00Z".into(),
        expires: "2099-01-01T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(std::time::SystemTime::now()), RepositoryStatus::Missing);
}

#[test]
fn test_status_stale_metadata() {
    let (root, _repos) = create_test_env("stale_meta");
    let now = std::time::SystemTime::now();
    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "stale".into(),
            name: "stale".into(),
            uuid: Uuid::new_v4(),
        },
        revision: 1,
        repository_root: root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2020-01-01T00:00:00Z".into(),
        expires: "2020-01-02T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(now), RepositoryStatus::StaleMetadata);
}

#[test]
fn test_status_audit_failed() {
    let (_root, repos) = create_test_env("audit_fail");
    let repo_root = init_repo(&repos, "bad-audit");
    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "bad-audit".into(),
            name: "bad-audit".into(),
            uuid: Uuid::new_v4(),
        },
        revision: 1,
        repository_root: repo_root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "FAIL".into(),
        last_sync: "2026-06-27T12:00:00Z".into(),
        expires: "2099-01-01T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(std::time::SystemTime::now()), RepositoryStatus::AuditFailed);
}

#[test]
fn test_status_stale_knowledge() {
    let (_root, repos) = create_test_env("stale_knowledge");
    let repo_root = init_repo(&repos, "sk");
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("sk", uuid, &repo_root, 5, vec![], "PASS");
    write_manifest(&repo_root, &manifest);

    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "sk".into(),
            name: "sk".into(),
            uuid,
        },
        revision: 3,  // manifest has revision 5 > cached 3
        repository_root: repo_root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2026-06-27T12:00:00Z".into(),
        expires: "2099-01-01T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(std::time::SystemTime::now()), RepositoryStatus::StaleKnowledge);
}

#[test]
fn test_status_sync_required() {
    let (_root, repos) = create_test_env("sync_required");
    let repo_root = init_repo(&repos, "sr");
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("sr", uuid, &repo_root, 1, vec![], "PASS");
    write_manifest(&repo_root, &manifest);

    let meta = CachedRepoMetadata {
        repository: RepoIdentity {
            id: "sr".into(),
            name: "sr".into(),
            uuid,
        },
        revision: 1,
        repository_root: repo_root.to_string_lossy().to_string(),
        knowledge: KnowledgeLocation { location: "dummy".into() },
        exports: vec![],
        audit: "PASS".into(),
        last_sync: "2026-06-27T11:00:00Z".into(),
        expires: "2099-01-01T00:00:00Z".into(),
        dependencies: Vec::new(),
    };
    assert_eq!(meta.status(std::time::SystemTime::now()), RepositoryStatus::SyncRequired);
}

// ── Resolution and cycle detection ────────────────────────────────────────

#[test]
fn test_resolve_simple_deps() {
    let (_root, repos) = create_test_env("resolve_simple");
    let dep_root = init_repo(&repos, "dep-a");
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("dep-a", uuid, &dep_root, 1, vec!["arch".into()], "PASS");
    write_manifest(&dep_root, &manifest);

    let config = SamgrahaConfig {
        repository: common::config::RepositoryConfig {
            dependencies: vec![DependencyConfig {
                name: "dep-a".into(),
                path: Some(dep_root.to_string_lossy().to_string()),
                required: true,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let root = dep_root.parent().unwrap().parent().unwrap().to_path_buf();
    let db = RegistryDb::open(&root).ok();
    let (resolved, unresolved) = KnowledgeResolver::resolve_dependency_graph(
        &config.repository.dependencies,
        &root,
        db.as_ref(),
        86400,
    );

    assert!(unresolved.is_empty(), "unexpected unresolved: {:?}", unresolved);
    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].available);
    assert_eq!(resolved[0].revision, 1);
}

#[test]
fn test_resolve_missing_dep() {
    let (_root, _repos) = create_test_env("resolve_missing");
    let missing_path = Path::new("Z:\\nonexistent\\dep").to_string_lossy().to_string();
    let config = SamgrahaConfig {
        repository: common::config::RepositoryConfig {
            dependencies: vec![DependencyConfig {
                name: "phantom".into(),
                path: Some(missing_path),
                required: true,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let root = std::env::temp_dir().join("samgraha_test").join("resolve_missing");
    let db = RegistryDb::open(&root).ok();
    let (resolved, unresolved) = KnowledgeResolver::resolve_dependency_graph(
        &config.repository.dependencies,
        &std::env::temp_dir(),
        db.as_ref(),
        86400,
    );

    assert_eq!(resolved.len(), 1);
    assert!(!resolved[0].available);
    assert!(!unresolved.is_empty(), "missing required dep should be in unresolved");
    assert!(unresolved[0].contains("phantom"));
}

#[test]
fn test_cycle_detection() {
    let (_root, repos) = create_test_env("cycle_detect");
    let dep_a_root = init_repo(&repos, "dep-a");
    let dep_b_root = init_repo(&repos, "dep-b");

    let uuid_a = Uuid::new_v4();
    let uuid_b = Uuid::new_v4();

    // dep-a manifest declares dependency on dep-b
    let mut ma = make_manifest("dep-a", uuid_a, &dep_a_root, 1, vec![], "PASS");
    ma.dependencies = vec!["dep-b".into()];
    write_manifest(&dep_a_root, &ma);

    // dep-b manifest declares dependency on dep-a (cycle!)
    let mut mb = make_manifest("dep-b", uuid_b, &dep_b_root, 1, vec![], "PASS");
    mb.dependencies = vec!["dep-a".into()];
    write_manifest(&dep_b_root, &mb);

    // Both deps in config so transitive path fallback resolves dep-b → finds cycle
    let config = SamgrahaConfig {
        repository: common::config::RepositoryConfig {
            dependencies: vec![
                DependencyConfig {
                    name: "dep-a".into(),
                    path: Some(dep_a_root.to_string_lossy().to_string()),
                    required: true,
                },
                DependencyConfig {
                    name: "dep-b".into(),
                    path: Some(dep_b_root.to_string_lossy().to_string()),
                    required: false,
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };

    let db = RegistryDb::open(&repos).ok();
    let (_resolved, unresolved) = KnowledgeResolver::resolve_dependency_graph(
        &config.repository.dependencies,
        &repos,
        db.as_ref(),
        86400,
    );

    assert!(!unresolved.is_empty(), "expected cycle error");
    let joined = unresolved.join(" ");
    assert!(joined.contains("dep-a"), "should mention dep-a, got: {}", joined);
    assert!(joined.contains("dep-b"), "should mention dep-b, got: {}", joined);
}

// ── Sync and TTL-aware ────────────────────────────────────────────────────

#[test]
fn test_sync_populates_cache() {
    let (root, repos) = create_test_env("sync_populate");
    let dep_root = init_repo(&repos, "sync-dep");
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("sync-dep", uuid, &dep_root, 42, vec!["arch".into()], "PASS");
    write_manifest(&dep_root, &manifest);

    let config = SamgrahaConfig {
        repository: common::config::RepositoryConfig {
            dependencies: vec![DependencyConfig {
                name: "sync-dep".into(),
                path: Some(dep_root.to_string_lossy().to_string()),
                required: false,
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let client = FileRegistryClient::new(&root);
    client.sync(&config).unwrap();

    let entries = client.list().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].repository.id, "sync-dep");
    assert_eq!(entries[0].revision, 42);

    assert!(!entries[0].last_sync.is_empty(), "last_sync should be set");
    assert!(!entries[0].expires.is_empty(), "expires should be set");
}

#[test]
fn test_ttl_from_config() {
    let (root, repos) = create_test_env("ttl_config");
    let dep_root = init_repo(&repos, "ttl-dep");
    let uuid = Uuid::new_v4();
    let manifest = make_manifest("ttl-dep", uuid, &dep_root, 1, vec![], "PASS");
    write_manifest(&dep_root, &manifest);

    let resolver_config = ResolverConfig {
        metadata_ttl: "7d".into(),
        ..Default::default()
    };

    let config = SamgrahaConfig {
        repository: common::config::RepositoryConfig {
            dependencies: vec![DependencyConfig {
                name: "ttl-dep".into(),
                path: Some(dep_root.to_string_lossy().to_string()),
                required: false,
            }],
            ..Default::default()
        },
        resolver: resolver_config,
        ..Default::default()
    };

    let client = FileRegistryClient::with_config(&root, &config.resolver);
    client.sync(&config).unwrap();

    let entries = client.list().unwrap();
    assert_eq!(entries.len(), 1);

    let expires = chrono::DateTime::parse_from_rfc3339(&entries[0].expires).unwrap();
    let expires_utc = expires.with_timezone(&chrono::Utc);
    let seven_days = chrono::Duration::days(7);
    let diff = expires_utc - chrono::Utc::now();
    assert!(diff > chrono::Duration::hours(160), "expected ~7d expiry, got {:?}", diff);
    assert!(diff < seven_days + chrono::Duration::minutes(5), "expiry too far out");
}
