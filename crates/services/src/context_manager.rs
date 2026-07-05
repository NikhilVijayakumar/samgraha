use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::config::SamgrahaConfig;

use crate::context::KnowledgeContext;

struct State {
    context: Option<KnowledgeContext>,
    connection_count: usize,
    inactive_since: Option<Instant>,
}

/// Owns the KnowledgeContext lifecycle: creation, validation, rebuild, and disposal.
/// Tracks active MCP connections so the context can outlive individual disconnects.
pub struct ContextManager {
    state: Mutex<State>,
    // ponytail: fixed 5-min dispose delay; add config field if operators need tuning
    dispose_after: Duration,
}

impl ContextManager {
    pub fn new(dispose_after: Duration) -> Self {
        Self {
            state: Mutex::new(State { context: None, connection_count: 0, inactive_since: None }),
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

    /// Rebuild context if absent or invalid. Swallows error (context stays None on failure).
    pub fn ensure(&self, root: &Path, config: &SamgrahaConfig) {
        let mut s = self.state.lock().unwrap();
        let needs_rebuild = s.context.as_ref().map_or(true, |c| !c.is_valid());
        if needs_rebuild {
            tracing::info!("Knowledge context stale or absent — rebuilding");
            match KnowledgeContext::create(root, config) {
                Ok(ctx) => {
                    tracing::info!("Knowledge context rebuilt: {} store(s)", ctx.store_count());
                    s.context = Some(ctx);
                }
                Err(e) => tracing::warn!("Context rebuild failed: {}", e),
            }
        }
    }

    /// Dispose context if all clients disconnected and dispose_after has elapsed.
    pub fn maybe_dispose(&self) {
        let mut s = self.state.lock().unwrap();
        if let Some(since) = s.inactive_since {
            if since.elapsed() > self.dispose_after {
                tracing::info!("Knowledge context disposed after inactivity");
                s.context = None;
                s.inactive_since = None;
            }
        }
    }

    /// Run a closure over the current context if one is present.
    pub fn with_context<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&KnowledgeContext) -> T,
    {
        let s = self.state.lock().unwrap();
        s.context.as_ref().map(f)
    }

    pub fn is_context_valid(&self) -> bool {
        self.with_context(|c| c.is_valid()).unwrap_or(false)
    }

    pub fn store_count(&self) -> usize {
        self.with_context(|c| c.store_count()).unwrap_or(0)
    }
}
