use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, Context};
use serde::Serialize;
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::audit::AuditReport;
use schemas::search::{SearchQuery, SearchResponse};
use schemas::config::SamgrahaConfig;
use schemas::document::Document;
use schemas::standard::StandardDefinition;
use registry::RegistryStore;
use services::registry::{ServiceRegistry, BoxedService};
use services::compilation::CompilationService;
use services::search::SearchService;
use services::audit::AuditService;
use audit::AuditFramework;
use standards::StandardRegistry;
use crate::context::RuntimeContext;
use crate::policy::RuntimePolicy;

pub struct KnowledgeRuntime {
    pub context: RuntimeContext,
    pub registry: Arc<RegistryStore>,
    pub services: ServiceRegistry,
    pub standard_registry: StandardRegistry,
    pub audit_framework: AuditFramework,
    pub policy: RuntimePolicy,
}

impl KnowledgeRuntime {
    pub fn new<P: AsRef<Path>>(
        root: P,
        config: SamgrahaConfig,
    ) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let registry_path = config
            .repository
            .root
            .clone()
            .unwrap_or_else(|| root.join("knowledge.db"));

        let registry = Arc::new(
            RegistryStore::open(&registry_path)
                .context("Failed to open knowledge registry")?,
        );

        let standard_registry = StandardRegistry::with_builtins();
        let audit_framework = AuditFramework::new();
        let services = ServiceRegistry::new();
        let policy = RuntimePolicy::default();

        let context = RuntimeContext::new(root, registry_path, config);

        Ok(Self {
            context,
            registry,
            services,
            standard_registry,
            audit_framework,
            policy,
        })
    }

    pub fn register_service(&mut self, service: BoxedService) {
        self.services.register(service);
    }

    pub fn register_audit_provider<F>(&mut self, name: &str, provider: F)
    where
        F: Fn(&[Document], &[schemas::standard::AuditRuleDef]) -> Vec<schemas::audit::AuditFinding>
            + Send + Sync
            + 'static,
    {
        self.audit_framework
            .register_provider(name, Arc::new(provider));
    }

    // --- Compilation ---

    pub fn compile(&self, request: &CompilationRequest) -> Result<CompilationResult> {
        let result = CompilationService::execute(
            &self.context.repository_root,
            &self.context.config,
            request,
            &self.standard_registry,
        )?;

        Ok(result)
    }

    // --- Search ---

    pub fn search(&self, query: &SearchQuery) -> Result<SearchResponse> {
        let docs = self.registry.get_all_documents()?;
        SearchService::search(&docs, query)
    }

    // --- Audit ---

    pub fn audit(
        &self,
        domain: Option<&str>,
        providers: &[String],
        documents: Option<&[Document]>,
    ) -> Result<AuditReport> {
        let docs = match documents {
            Some(d) => d.to_vec(),
            None => self.registry.get_all_documents()?,
        };

        let standards: Vec<StandardDefinition> = self.standard_registry.all()
            .into_iter()
            .cloned()
            .collect();

        let result = AuditService::execute(
            &self.audit_framework,
            domain,
            &docs,
            &standards,
            providers,
        )?;

        self.registry.clear_audit_results()?;
        self.registry.insert_audit_findings(&result.findings)?;

        Ok(result)
    }

    // --- Document Retrieval ---

    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        self.registry.get_document(id)
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        self.registry.get_document_by_path(path)
    }

    pub fn get_all_documents(&self) -> Result<Vec<Document>> {
        self.registry.get_all_documents()
    }

    // --- Info ---

    pub fn info(&self) -> RuntimeInfo {
        let repo_name = self.context.repository_name();
        let doc_count = self.registry.document_count().unwrap_or(0);
        RuntimeInfo {
            repository: repo_name,
            registry_path: self.context.registry_path.display().to_string(),
            document_count: doc_count,
            standards: self.standard_registry.domains().into_iter().map(|s| s.to_string()).collect(),
            services: self.services.all().iter().map(|s| s.name().to_string()).collect(),
            policy: self.policy.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeInfo {
    pub repository: String,
    pub registry_path: String,
    pub document_count: usize,
    pub standards: Vec<String>,
    pub services: Vec<String>,
    pub policy: RuntimePolicy,
}
