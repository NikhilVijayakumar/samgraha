#[cfg(test)]
mod integration_tests {
    use std::collections::HashMap;
    use schemas::audit::{QualityGate, ReadinessAssessment};
    use schemas::compilation::{CompilationRequest, CompilationScope};
    use schemas::search::{SearchQuery, RetrievalLevel};
    use schemas::config::SamgrahaConfig;
    use registry::RegistryStore;
    use runtime::KnowledgeRuntime;
    use services::search::SearchService;

    #[test]
    fn test_registry_open_and_migrate() {
        let store = RegistryStore::open_in_memory().unwrap();
        let meta = store.check_integrity().unwrap();
        assert_eq!(meta.document_count, 0);
    }

    #[test]
    fn test_registry_insert_and_query() {
        let store = RegistryStore::open_in_memory().unwrap();
        let doc = crate::fixtures::sample_document(1, "architecture", "Test", "# Test\n\nContent");
        store.insert_document(&doc).unwrap();
        assert_eq!(store.document_count().unwrap(), 1);
        let retrieved = store.get_document(1).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test");
    }

    #[test]
    fn test_registry_search() {
        let store = RegistryStore::open_in_memory().unwrap();
        let docs = crate::fixtures::sample_documents();
        for doc in &docs {
            store.insert_document(doc).unwrap();
        }

        let query = SearchQuery {
            query: "compilation".to_string(),
            level: RetrievalLevel::Metadata,
            max_results: 10,
            ..Default::default()
        };

        let docs = store.get_all_documents().unwrap();
        let results = SearchService::search(&docs, &query).unwrap();
        assert!(!results.results.is_empty());
        assert!(results.results[0].title.contains("Knowledge"));
    }

    #[test]
    fn test_deterministic_audit() {
        let store = RegistryStore::open_in_memory().unwrap();
        let docs = crate::fixtures::sample_documents();
        for doc in &docs {
            store.insert_document(doc).unwrap();
        }

        let config = SamgrahaConfig::default();
        let root = std::env::current_dir().unwrap();
        let mut runtime = KnowledgeRuntime::new(&root, config).unwrap();

        runtime.register_audit_provider("deterministic", |docs, rules| {
            audit::DeterministicAuditProvider::execute(docs, rules)
        });

        let report = runtime.audit(None, &["deterministic".to_string()], None).unwrap();
        assert!(!report.findings.is_empty() || report.score.overall > 0.0);
    }

    #[test]
    fn test_runtime_info() {
        let config = SamgrahaConfig::default();
        let root = std::env::current_dir().unwrap();
        let runtime = KnowledgeRuntime::new(&root, config).unwrap();
        let info = runtime.info();
        assert!(!info.repository.is_empty());
        assert_eq!(info.repository, "tests");
    }

    #[test]
    fn test_compilation_service() {
        let config = SamgrahaConfig::default();
        let root = std::env::current_dir().unwrap();
        let runtime = KnowledgeRuntime::new(&root, config).unwrap();

        // Test that compilation discovers docs/ directory
        let request = CompilationRequest {
            scope: CompilationScope::Repository,
            force: false,
            watch: false,
        };
        let result = runtime.compile(&request).unwrap();
        assert_eq!(result.success, true);
    }

    #[test]
    fn test_runtime_context_basics() {
        let config = SamgrahaConfig::default();
        let root = std::env::current_dir().unwrap();
        let runtime = KnowledgeRuntime::new(&root, config).unwrap();

        let name = runtime.context.repository_name();
        assert_eq!(name, "tests");
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

    #[test]
    fn test_quality_gate_passes() {
        use schemas::audit::AuditScore;
        let score = AuditScore {
            overall: 95.0,
            categories: HashMap::new(),
            documents_checked: 10,
            documents_passed: 10,
            findings_count: 0,
        };
        let report = schemas::audit::AuditReport {
            id: "test".into(),
            domain: None,
            timestamp: "now".into(),
            provider: "test".into(),
            score,
            findings: vec![],
            readiness: ReadinessAssessment::Production,
            metadata: HashMap::new(),
        };
        let gate = QualityGate {
            enabled: true,
            min_score: Some(90.0),
            min_readiness: Some(ReadinessAssessment::Implementation),
            required_domains: vec![],
        };
        assert!(audit::AuditFramework::check_quality_gate(&report, &gate).unwrap());
    }

    #[test]
    fn test_provider_integration() {
        use providers::traits::EnrichmentProvider;
        let provider = providers::RuleBasedProvider::new();
        let doc = crate::fixtures::sample_document(1, "architecture", "Test", "# Test\n\nContent here");
        let summary = provider.summarize(&doc).unwrap();
        assert!(!summary.summary.is_empty());
    }
}
