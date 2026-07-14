mod fixtures;

use common::config::SamgrahaConfig;
use schemas::compilation::{CompilationRequest, CompilationScope};
use services::KnowledgeRuntime;

#[test]
fn test_compilation_service() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let mut config = SamgrahaConfig::default();
    config.repository.root = Some(tmp_dir.path().join("knowledge.db"));
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    let request = CompilationRequest {
        scope: CompilationScope::Repository,
        force: false,
        watch: false,
    };
    let result = runtime.compile(&request).unwrap();
    assert_eq!(result.success, true);
}

#[test]
fn compile_populates_repository_metadata() {
    // Regression: `repository_metadata` (Product Guide Audit Phase 1.5) was
    // created by the V30 migration but nothing ever wrote to it — this
    // drives a real compile and checks the table is actually populated
    // afterward, not just that the schema exists.
    let tmp_dir = tempfile::tempdir().unwrap();
    let mut config = SamgrahaConfig::default();
    config.repository.root = Some(tmp_dir.path().join("knowledge.db"));
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    let request = CompilationRequest {
        scope: CompilationScope::Repository,
        force: true,
        watch: false,
    };
    let result = runtime.compile(&request).unwrap();
    assert!(result.success);

    let meta = runtime.registry.get_repository_metadata().unwrap();
    assert!(meta.contains_key("source_dir"));
    assert!(meta.contains_key("compiled_at"));
    assert!(meta.contains_key("repo_root"));
    assert!(meta.contains_key("uuid"));
}

#[test]
fn audit_registers_default_providers() {
    // Regression: KnowledgeRuntime::new() previously never registered any
    // audit providers — every provider lookup in AuditFramework::execute
    // silently missed, so `audit` returned 0 findings / 100 score for every
    // domain, every repo, every time (via MCP; the CLI masked it by
    // registering providers locally right before calling audit()).
    let tmp_dir = tempfile::tempdir().unwrap();
    let mut config = SamgrahaConfig::default();
    config.repository.root = Some(tmp_dir.path().join("knowledge.db"));
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    let request = CompilationRequest {
        scope: CompilationScope::Repository,
        force: false,
        watch: false,
    };
    runtime.compile(&request).unwrap();

    let report = runtime
        .audit(Some("architecture"), &["deterministic".to_string()], None)
        .unwrap();
    assert_eq!(report.provider, "deterministic");
}

#[test]
fn test_standard_registry_db_backed() {
    use standards::StandardRegistry;
    let tmp = fixtures::create_test_standards_db();
    let registry = StandardRegistry::from_standards_db_and_overrides(&tmp).unwrap();
    let domains = registry.domains();
    assert!(domains.contains(&"architecture"), "expected architecture domain, got {:?}", domains);
    assert!(domains.contains(&"feature"), "expected feature domain, got {:?}", domains);
    assert!(domains.contains(&"vision"), "expected vision domain, got {:?}", domains);
    assert!(domains.len() >= 3, "expected at least 3 domains, got {}", domains.len());

    // Verify architecture has rules.
    let arch = registry.get("architecture", "1.0.0").expect("architecture standard not found");
    assert!(!arch.audit_rules.is_empty(), "architecture should have rules");
    assert!(!arch.required_sections.is_empty(), "architecture should have sections");

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_knowledge_system_compilation() {
    use common::config::RepositoryKind;
    let tmp_dir = tempfile::tempdir().unwrap();
    let mut config = SamgrahaConfig::default();
    config.repository.kind = RepositoryKind::Knowledge;
    config.repository.root = Some(tmp_dir.path().join("knowledge.db"));
    let root = std::env::current_dir().unwrap();
    let runtime = KnowledgeRuntime::new(&root, config).unwrap();

    let request = CompilationRequest {
        scope: CompilationScope::Repository,
        force: false,
        watch: false,
    };
    let result = runtime.compile(&request).unwrap();
    assert_eq!(result.success, true);
}
