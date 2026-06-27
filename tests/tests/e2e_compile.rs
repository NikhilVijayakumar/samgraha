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
fn test_standard_registry_builtins() {
    use standards::StandardRegistry;
    let registry = StandardRegistry::with_builtins();
    let domains = registry.domains();
    assert!(domains.contains(&"architecture"));
    assert!(domains.contains(&"feature"));
    assert!(domains.contains(&"engineering"));
    assert!(domains.len() >= 10);
}
