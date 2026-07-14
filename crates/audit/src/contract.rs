use common::config::ResolvedContract;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Instant, SystemTime};

/// Runs (or verifies, without running) a repository-declared Pipeline Contract
/// (`samgraha.toml [pipelines.build]` etc.). Deliberately not named anything
/// with `Pipeline` in it and deliberately a separate module from `pipeline.rs`
/// — that file owns `Pipeline`/`PipelineContext`/`PipelineKind` for the 5
/// audit types (Build, Security, Consistency, Coverage, Dependency); this is
/// an unrelated concept (user-declared, executable commands).
pub struct ContractRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ArtifactStatus {
    pub path: PathBuf,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub artifacts: Vec<ArtifactStatus>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub matched_expected_exit_code: bool,
}

impl ContractRunner {
    /// Verify-only mode: never executes anything. Classifies each declared
    /// artifact as Fresh / Stale / Unknown relative to `source_root`.
    pub fn verify(resolved: &ResolvedContract, source_root: &Path) -> VerifyReport {
        let source_mtime = newest_mtime_under(source_root, &[]);
        let artifacts = resolved
            .artifacts
            .iter()
            .map(|path| ArtifactStatus {
                path: path.clone(),
                freshness: classify_freshness(path, source_mtime),
            })
            .collect();
        VerifyReport { artifacts }
    }

    /// Refuses to run unless `resolved.working_directory` is `project_root`
    /// itself or a descendant of it. This is the concrete mechanism behind
    /// the trust-boundary invariant: a Pipeline Contract can only ever
    /// execute inside the repository that declared it, never escape to a
    /// dependency repo or anywhere else on disk via a crafted
    /// `working_directory`.
    pub fn execute(
        resolved: &ResolvedContract,
        project_root: &Path,
        timeout_secs: Option<i64>,
    ) -> Result<ExecutionResult, String> {
        if resolved.command.is_empty() {
            return Err("Contract declares no command".to_string());
        }
        if !is_within(&resolved.working_directory, project_root) {
            return Err(format!(
                "Refusing to execute: working_directory '{}' escapes project root '{}'",
                resolved.working_directory.display(),
                project_root.display()
            ));
        }

        let mut cmd = std::process::Command::new(&resolved.command[0]);
        cmd.args(&resolved.command[1..]);
        cmd.current_dir(&resolved.working_directory);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", resolved.command[0], e))?;

        let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
        let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
        let stdout_handle = std::thread::spawn(move || {
            let mut buf = String::new();
            let _ = stdout_pipe.read_to_string(&mut buf);
            buf
        });
        let stderr_handle = std::thread::spawn(move || {
            let mut buf = String::new();
            let _ = stderr_pipe.read_to_string(&mut buf);
            buf
        });

        let start = Instant::now();
        // ponytail: 50ms poll loop, no process-group/tree kill — sufficient for
        // a direct child; a command that forks its own children can outlive
        // the timeout kill. Container sandboxing (deferred to v2) would close this.
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if let Some(t) = timeout_secs {
                if start.elapsed().as_secs() as i64 > t {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("Command timed out after {}s", t));
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };

        let stdout = stdout_handle.join().unwrap_or_default();
        let stderr = stderr_handle.join().unwrap_or_default();
        let exit_code = status.code().unwrap_or(-1);

        Ok(ExecutionResult {
            exit_code,
            stdout,
            stderr,
            matched_expected_exit_code: exit_code == resolved.success_exit_code,
        })
    }
}

fn is_within(candidate: &Path, root: &Path) -> bool {
    // Prevent traversal attacks like /project_root/.. by canonicalizing if possible.
    // If canonicalization fails (e.g. path doesn't exist), fall back to basic lexical
    // checks but strictly reject any `..` components to prevent escaping.
    let candidate_canon = std::fs::canonicalize(candidate);
    let root_canon = std::fs::canonicalize(root);

    if let (Ok(c), Ok(r)) = (candidate_canon, root_canon) {
        c.starts_with(&r)
    } else {
        if candidate.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return false;
        }
        let candidate = std::path::absolute(candidate).unwrap_or_else(|_| candidate.to_path_buf());
        let root = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
        candidate.starts_with(&root)
    }
}

fn classify_freshness(artifact: &Path, source_mtime: Option<SystemTime>) -> Freshness {
    let Ok(meta) = std::fs::metadata(artifact) else {
        return Freshness::Stale; // missing artifact — definitely stale
    };
    let Ok(artifact_mtime) = meta.modified() else {
        return Freshness::Unknown;
    };
    match source_mtime {
        Some(source_mtime) => {
            if artifact_mtime >= source_mtime {
                Freshness::Fresh
            } else {
                Freshness::Stale
            }
        }
        None => Freshness::Unknown,
    }
}

/// Newest modification time under `dir`, skipping any path component that
/// matches an entry in `exclude` by name.
/// ponytail: unbounded recursive walk, no depth/file-count cap — fine for a
/// single repo's source tree today; add a cap if this ever runs over
/// something large enough to matter.
fn newest_mtime_under(dir: &Path, exclude: &[String]) -> Option<SystemTime> {
    let mut newest: Option<SystemTime> = None;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if exclude.iter().any(|e| e == name.as_ref()) {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                stack.push(path);
            } else if let Ok(modified) = meta.modified() {
                if newest.map_or(true, |n| modified > n) {
                    newest = Some(modified);
                }
            }
        }
    }
    newest
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("samgraha-contract-test-{}-{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_artifact_is_stale() {
        let root = temp_dir();
        let status = classify_freshness(&root.join("does-not-exist.bin"), Some(SystemTime::now()));
        assert_eq!(status, Freshness::Stale);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_newer_than_source_is_fresh() {
        let root = temp_dir();
        let artifact = root.join("out.bin");
        std::fs::write(&artifact, b"x").unwrap();
        let source_mtime = SystemTime::now() - std::time::Duration::from_secs(60);
        let status = classify_freshness(&artifact, Some(source_mtime));
        assert_eq!(status, Freshness::Fresh);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn artifact_older_than_source_is_stale() {
        let root = temp_dir();
        let artifact = root.join("out.bin");
        std::fs::write(&artifact, b"x").unwrap();
        let source_mtime = SystemTime::now() + std::time::Duration::from_secs(60);
        let status = classify_freshness(&artifact, Some(source_mtime));
        assert_eq!(status, Freshness::Stale);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn no_artifacts_declared_is_unknown_not_stale() {
        // deploy-shaped contract: empty artifacts list
        let root = temp_dir();
        let resolved = ResolvedContract {
            command: vec!["echo".into()],
            working_directory: root.clone(),
            artifacts: vec![],
            success_exit_code: 0,
        };
        let report = ContractRunner::verify(&resolved, &root);
        assert!(report.artifacts.is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_refuses_working_directory_outside_project_root() {
        let project_root = temp_dir();
        let outside = temp_dir();
        let resolved = ResolvedContract {
            command: vec!["echo".into(), "hi".into()],
            working_directory: outside.clone(),
            artifacts: vec![],
            success_exit_code: 0,
        };
        let result = ContractRunner::execute(&resolved, &project_root, Some(5));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escapes"));
        let _ = std::fs::remove_dir_all(&project_root);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn execute_refuses_empty_command() {
        let root = temp_dir();
        let resolved = ResolvedContract {
            command: vec![],
            working_directory: root.clone(),
            artifacts: vec![],
            success_exit_code: 0,
        };
        let result = ContractRunner::execute(&resolved, &root, Some(5));
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn execute_runs_command_and_captures_output() {
        let root = temp_dir();
        let (cmd, args): (&str, &[&str]) = if cfg!(windows) {
            ("cmd", &["/C", "echo hello"])
        } else {
            ("echo", &["hello"])
        };
        let mut command = vec![cmd.to_string()];
        command.extend(args.iter().map(|s| s.to_string()));
        let resolved = ResolvedContract {
            command,
            working_directory: root.clone(),
            artifacts: vec![],
            success_exit_code: 0,
        };
        let result = ContractRunner::execute(&resolved, &root, Some(10)).unwrap();
        assert!(result.stdout.contains("hello"));
        assert!(result.matched_expected_exit_code);
        let _ = std::fs::remove_dir_all(&root);
    }
}
