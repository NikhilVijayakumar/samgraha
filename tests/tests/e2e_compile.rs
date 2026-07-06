mod fixtures;

use common::config::SamgrahaConfig;
use schemas::compilation::{CompilationRequest, CompilationScope};
use services::KnowledgeRuntime;

#[test]
fn test_compilation_service() {
    let config = SamgrahaConfig::default();
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
    let config = SamgrahaConfig::default();
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
fn test_standard_registry_builtins() {
    use standards::StandardRegistry;
    let registry = StandardRegistry::with_builtins();
    let domains = registry.domains();
    assert!(domains.contains(&"architecture"));
    assert!(domains.contains(&"feature"));
    assert!(domains.contains(&"engineering"));
    assert!(domains.len() >= 10);
}
