use std::collections::HashMap;
use std::sync::Arc;

pub type BoxedService = Arc<dyn KnowledgeService + Send + Sync>;

pub trait KnowledgeService {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

pub struct ServiceRegistry {
    services: HashMap<String, BoxedService>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn register(&mut self, service: BoxedService) {
        self.services.insert(service.name().to_string(), service);
    }

    pub fn get(&self, name: &str) -> Option<&BoxedService> {
        self.services.get(name)
    }

    pub fn all(&self) -> Vec<&BoxedService> {
        self.services.values().collect()
    }

    pub fn has(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
