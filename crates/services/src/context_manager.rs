use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::config::SamgrahaConfig;

use crate::context::KnowledgeContext;

struct State {
    contexts: HashMap<String, KnowledgeContext>,
    active_name: Option<String>,
    connection_count: usize,
    inactive_since: Option<Instant>,
}

/// Owns one or more named KnowledgeContexts. Tracks active MCP connections
/// so contexts survive individual disconnects. Active context serves all queries.
pub struct ContextManager {
    state: Mutex<State>,
    // ponytail: fixed 5-min dispose delay; add config field if operators need tuning
    dispose_after: Duration,
}

impl ContextManager {
    pub fn new(dispose_after: Duration) -> Self {
        Self {
            state: Mutex::new(State {
                contexts: HashMap::new(),
                active_name: None,
                connection_count: 0,
                inactive_since: None,
            }),
            dispose_after,
        }
    }

    /// Signal a new MCP client connection. Clears inactivity timer.
    pub fn connect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connection_count += 1;
        s.inactive_since = None;
    }

    /// Signal an MCP client disconnect. Starts inactivity timer when count reaches zero.
    pub fn disconnect(&self) {
        let mut s = self.state.lock().unwrap();
        s.connection_count = s.connection_count.saturating_sub(1);
        if s.connection_count == 0 {
            s.inactive_since = Some(Instant::now());
        }
    }

    /// Ensure named context is present and valid; set it as active if no active context exists.
    /// Swallows rebuild errors (context stays absent on failure).
    pub fn ensure(&self, name: &str, root: &Path, config: &SamgrahaConfig) {
        let mut s = self.state.lock().unwrap();
        let needs_rebuild = s.contexts.get(name).map_or(true, |c| !c.is_valid());
        if needs_rebuild {
            tracing::info!("Knowledge context '{}' stale or absent — rebuilding", name);
            match KnowledgeContext::create(root, config) {
                Ok(ctx) => {
                    tracing::info!("Knowledge context '{}' rebuilt: {} store(s)", name, ctx.store_count());
                    s.contexts.insert(name.to_string(), ctx);
                }
                Err(e) => tracing::warn!("Context '{}' rebuild failed: {}", name, e),
            }
        }
        if s.active_name.is_none() && s.contexts.contains_key(name) {
            s.active_name = Some(name.to_string());
        }
    }

    /// Switch the active context to an already-loaded named context.
    /// Returns false if the name is not loaded.
    pub fn activate(&self, name: &str) -> bool {
        let mut s = self.state.lock().unwrap();
        if s.contexts.contains_key(name) {
            s.active_name = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Names of all loaded contexts, active context first.
    pub fn context_names(&self) -> Vec<String> {
        let s = self.state.lock().unwrap();
        let mut names: Vec<String> = s.contexts.keys().cloned().collect();
        if let Some(ref active) = s.active_name {
            names.retain(|n| n != active);
            names.insert(0, active.clone());
        }
        names
    }

    /// Dispose all contexts when inactive longer than dispose_after.
    pub fn maybe_dispose(&self) {
        let mut s = self.state.lock().unwrap();
        if let Some(since) = s.inactive_since {
            if since.elapsed() > self.dispose_after {
                tracing::info!("All knowledge contexts disposed after inactivity");
                s.contexts.clear();
                s.active_name = None;
                s.inactive_since = None;
            }
        }
    }

    /// Run a closure over the active context, if one is present.
    pub fn with_context<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&KnowledgeContext) -> T,
    {
        let s = self.state.lock().unwrap();
        s.active_name.as_ref()
            .and_then(|name| s.contexts.get(name))
            .map(f)
    }

    pub fn is_context_valid(&self) -> bool {
        self.with_context(|c| c.is_valid()).unwrap_or(false)
    }

    pub fn store_count(&self) -> usize {
        self.with_context(|c| c.store_count()).unwrap_or(0)
    }

    pub fn active_name(&self) -> Option<String> {
        self.state.lock().unwrap().active_name.clone()
    }
}
