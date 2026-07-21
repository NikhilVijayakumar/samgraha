use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Locates `knowledge-hub-loader.py`: `SAMGRAHA_KNOWLEDGE_HUB_LOADER` env var
/// override first, then binary-adjacent `schema/knowledge-hub/` at one or two
/// directories up (dev vs. installed layout). Shared by CLI `knowledge
/// publish` and MCP `register_standard` — both invoke the same Python
/// loader the same way (Optimization 8).
pub fn resolve_knowledge_hub_loader() -> Result<PathBuf> {
    let exe_dir = std::env::current_exe()?
        .parent().ok_or_else(|| anyhow::anyhow!("Cannot determine binary directory"))?
        .to_path_buf();
    let env_loader = std::env::var("SAMGRAHA_KNOWLEDGE_HUB_LOADER").ok().map(PathBuf::from);
    Ok(env_loader.filter(|p| p.exists()).unwrap_or_else(|| {
        let p1 = exe_dir.join("..").join("schema").join("knowledge-hub").join("knowledge-hub-loader.py");
        if p1.exists() {
            p1
        } else {
            exe_dir.join("..").join("..").join("schema").join("knowledge-hub").join("knowledge-hub-loader.py")
        }
    }))
}

/// Runs the Python knowledge-hub-loader once against a single knowledge-hub
/// directory, writing into `local_db`.
pub fn run_knowledge_hub_loader(
    loader: &Path,
    local_db: &Path,
    hub_path: &Path,
    system: Option<&str>,
    layout: Option<&Path>,
    dry_run: bool,
) -> Result<String> {
    let mut cmd = common::env::python_command();
    cmd.arg(loader)
        .arg("--db").arg(local_db)
        .arg("--knowledge-hub").arg(hub_path);
    if let Some(system) = system {
        cmd.arg("--system").arg(system);
    }
    if let Some(layout) = layout {
        cmd.arg("--layout").arg(layout);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }
    let output = cmd.output().map_err(|e| anyhow::anyhow!("Failed to run loader: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Loader failed: {}", stderr);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Guard against the skip-sync clobber: refuse to push `local_db` over
/// `global_db` when the global store already contains systems that the
/// local db doesn't have. Without this, a repo that never ran
/// `sync_standards` first silently wipes every other repo's systems.
///
/// Returns `Ok(())` when the push is safe (local is a superset of global,
/// or global doesn't exist yet). Returns `Err` with a message listing the
/// systems that would be lost.
pub fn check_push_safe(local_db: &Path, global_db: &Path) -> Result<()> {
    if !global_db.exists() {
        return Ok(()); // no global store yet, nothing to lose
    }

    let global_conn = rusqlite::Connection::open(global_db)?;
    let local_conn = rusqlite::Connection::open(local_db)?;

    let global_names: HashSet<String> = {
        let mut stmt = global_conn.prepare("SELECT name FROM systems")?;
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect()
    };

    let local_names: HashSet<String> = {
        let mut stmt = local_conn.prepare("SELECT name FROM systems")?;
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().collect()
    };

    let missing: Vec<&str> = global_names.difference(&local_names)
        .map(|s| s.as_str())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        bail!(
            "Push refused: the global standards.db contains systems not in your \
             local db: {}. Run sync_standards first to pull them before pushing.",
            missing.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_knowledge_hub_loader_env_override_takes_precedence() {
        let tmp = std::env::temp_dir().join(format!("samgraha-loader-override-test-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let fake_loader = tmp.join("fake-loader.py");
        std::fs::write(&fake_loader, "# fake").unwrap();

        // SAFETY: test-only, single-threaded within this process's test binary
        // for this specific env var; no other test reads/writes it.
        unsafe { std::env::set_var("SAMGRAHA_KNOWLEDGE_HUB_LOADER", &fake_loader) };
        let resolved = resolve_knowledge_hub_loader().unwrap();
        unsafe { std::env::remove_var("SAMGRAHA_KNOWLEDGE_HUB_LOADER") };

        assert_eq!(resolved, fake_loader);
        std::fs::remove_dir_all(&tmp).ok();
    }
}
