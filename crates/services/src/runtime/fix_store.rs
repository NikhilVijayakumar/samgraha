use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Context;
use audit_crate::fix::orchestrator::FixStore;
pub use audit_crate::fix::types::*;
use registry::RegistryStore;

/// Thread-safe wrapper around RegistryStore for the FixStore trait.
///
/// Opens a dedicated SQLite connection to the same database so the adapter
/// satisfies `Send + Sync` (rusqlite::Connection is `Send` but not `Sync`,
/// so we wrap it in `Mutex`).
pub struct RegistryFixStore {
    store: Mutex<RegistryStore>,
}

impl RegistryFixStore {
    pub fn new(db_path: PathBuf) -> anyhow::Result<Self> {
        let store = RegistryStore::open(&db_path)
            .with_context(|| format!("Failed to open fix store at {}", db_path.display()))?;
        Ok(Self { store: Mutex::new(store) })
    }
}

impl FixStore for RegistryFixStore {
    fn insert_session(&self, session: &FixSession) -> anyhow::Result<i64> {
        self.store.lock().unwrap().insert_fix_session(session)
    }

    fn update_session(&self, session: &FixSession) -> anyhow::Result<()> {
        self.store.lock().unwrap().update_fix_session(session)
    }

    fn get_session(&self, id: i64) -> anyhow::Result<Option<FixSession>> {
        self.store.lock().unwrap().get_fix_session(id)
    }

    fn insert_plan(&self, plan: &FixPlan) -> anyhow::Result<i64> {
        self.store.lock().unwrap().insert_fix_plan(plan)
    }

    fn get_plan(&self, id: i64) -> anyhow::Result<Option<FixPlan>> {
        self.store.lock().unwrap().get_fix_plan(id)
    }

    fn update_plan_status(&self, plan_id: i64, status: &FixPlanStatus) -> anyhow::Result<()> {
        self.store.lock().unwrap().update_fix_plan_status(plan_id, status)
    }

    fn insert_step(&self, step: &PlanStep) -> anyhow::Result<i64> {
        self.store.lock().unwrap().insert_fix_plan_step(step)
    }

    fn update_step(&self, step: &PlanStep) -> anyhow::Result<()> {
        self.store.lock().unwrap().update_fix_plan_step(step)
    }

    fn get_steps(&self, plan_id: i64) -> anyhow::Result<Vec<PlanStep>> {
        self.store.lock().unwrap().get_fix_plan_steps(plan_id)
    }

    fn insert_attempt(&self, attempt: &FixAttempt) -> anyhow::Result<i64> {
        self.store.lock().unwrap().insert_fix_attempt(attempt)
    }

    fn get_attempts(&self, session_id: i64) -> anyhow::Result<Vec<FixAttempt>> {
        self.store.lock().unwrap().get_fix_attempts(session_id)
    }
}
