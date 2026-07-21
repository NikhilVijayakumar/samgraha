use anyhow::{Context, Result};
use std::path::Path;

/// Default exclusion patterns for standard asset sync — `__pycache__`
/// directories and `.pyc` bytecode files. Always applied on top of any
/// caller-supplied excludes.
pub const DEFAULT_EXCLUDES: &[&str] = &["**/__pycache__/**", "**/*.pyc"];

/// Recursively copy `src` into `dest`, applying glob-based exclusion
/// patterns. Returns the number of files copied.
///
/// - Walks all subdirectories (unlike the old flat `read_dir` copies).
/// - Skips files/dirs matching any pattern in `exclude` (matched against
///   the path relative to `src`, using `/` separators).
/// - Skips `__pycache__` dirs and `.pyc` files unconditionally (on top of
///   caller-supplied patterns).
/// - Preserves directory structure.
/// - Does NOT perform atomic rename — callers that need atomicity wrap
///   this in a temp-dir + `fs::rename` themselves (see §3.2 of the asset
///   sync proposal).
pub fn copy_dir_recursive(src: &Path, dest: &Path, exclude: &[&str]) -> Result<usize> {
    let mut count = 0;
    // Merge caller-supplied excludes with unconditional defaults.
    let mut all_excludes: Vec<&str> = DEFAULT_EXCLUDES.iter().copied().collect();
    all_excludes.extend(exclude.iter().copied());
    copy_dir_recursive_inner(src, dest, "", &all_excludes, &mut count)?;
    Ok(count)
}

fn copy_dir_recursive_inner(
    src: &Path,
    dest: &Path,
    rel_prefix: &str,
    exclude: &[&str],
    count: &mut usize,
) -> Result<()> {
    std::fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create dir {}", dest.display()))?;

    for entry in std::fs::read_dir(src)
        .with_context(|| format!("Failed to read dir {}", src.display()))?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_type = entry.file_type()?;

        // Build the path relative to the copy root for glob matching.
        let rel_str = if rel_prefix.is_empty() {
            file_name.to_string_lossy().to_string()
        } else {
            format!("{}/{}", rel_prefix, file_name.to_string_lossy())
        };

        // Check exclusion patterns
        if should_exclude(&rel_str, exclude) {
            continue;
        }

        let dest_path = dest.join(&file_name);

        if file_type.is_dir() {
            copy_dir_recursive_inner(&entry.path(), &dest_path, &rel_str, exclude, count)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &dest_path)
                .with_context(|| format!(
                    "Failed to copy {} -> {}",
                    entry.path().display(),
                    dest_path.display()
                ))?;
            *count += 1;
        }
        // Symlinks are intentionally skipped — standards should not ship them.
    }
    Ok(())
}

/// Check whether `rel_path` (forward-slash-separated, relative to the
/// copy base) matches any of the exclusion patterns.
fn should_exclude(rel_path: &str, exclude: &[&str]) -> bool {
    for pattern in exclude {
        if crate::glob::matches_glob(pattern, rel_path) {
            return true;
        }
    }
    false
}

/// Atomically copy `src` into `dest` using a sibling temp directory and
/// `fs::rename`. If the copy fails partway, the previous `dest` tree
/// remains untouched. Returns the number of files copied.
pub fn copy_dir_atomic(src: &Path, dest: &Path, exclude: &[&str]) -> Result<usize> {
    let parent = dest
        .parent()
        .with_context(|| format!("No parent for {}", dest.display()))?;
    std::fs::create_dir_all(parent)?;

    let uuid = uuid::Uuid::new_v4();
    let tmp_name = format!(".tmp-sync-{}", uuid);
    let tmp_dir = parent.join(&tmp_name);

    // Clean up any leftover temp dir from a previous failed run.
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }

    let result = copy_dir_recursive(src, &tmp_dir, exclude);

    match result {
        Ok(count) => {
            // Atomic rename — same filesystem, so this is a metadata
            // operation on both Windows (MoveFileEx) and Unix (rename(2)).
            if dest.exists() {
                std::fs::remove_dir_all(dest)?;
            }
            std::fs::rename(&tmp_dir, dest).with_context(|| format!(
                "Failed to atomically rename {} -> {}",
                tmp_dir.display(),
                dest.display()
            ))?;
            Ok(count)
        }
        Err(e) => {
            // Best-effort cleanup of the partial temp dir.
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("samgraha-fsync-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copies_flat_files() {
        let src = scratch("flat");
        let dest = scratch("flat-dest");
        fs::write(src.join("a.py"), "print('a')").unwrap();
        fs::write(src.join("b.py"), "print('b')").unwrap();

        let n = copy_dir_recursive(&src, &dest, &[]).unwrap();
        assert_eq!(n, 2);
        assert!(dest.join("a.py").exists());
        assert!(dest.join("b.py").exists());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn copies_nested_dirs() {
        let src = scratch("nested");
        let dest = scratch("nested-dest");
        fs::create_dir_all(src.join("sub/deep")).unwrap();
        fs::write(src.join("top.py"), "").unwrap();
        fs::write(src.join("sub/mid.py"), "").unwrap();
        fs::write(src.join("sub/deep/leaf.py"), "").unwrap();

        let n = copy_dir_recursive(&src, &dest, &[]).unwrap();
        assert_eq!(n, 3);
        assert!(dest.join("top.py").exists());
        assert!(dest.join("sub/mid.py").exists());
        assert!(dest.join("sub/deep/leaf.py").exists());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn excludes_pycache() {
        let src = scratch("pycache");
        let dest = scratch("pycache-dest");
        fs::create_dir_all(src.join("__pycache__")).unwrap();
        fs::write(src.join("__pycache__/a.pyc"), "").unwrap();
        fs::write(src.join("real.py"), "").unwrap();

        let n = copy_dir_recursive(&src, &dest, &[]).unwrap();
        assert_eq!(n, 1);
        assert!(dest.join("real.py").exists());
        assert!(!dest.join("__pycache__").exists());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn excludes_custom_glob() {
        let src = scratch("exclude");
        let dest = scratch("exclude-dest");
        fs::create_dir_all(src.join("archive")).unwrap();
        fs::write(src.join("archive/old.py"), "").unwrap();
        fs::write(src.join("active.py"), "").unwrap();

        let n = copy_dir_recursive(&src, &dest, &["archive/**"]).unwrap();
        assert_eq!(n, 1);
        assert!(dest.join("active.py").exists());
        assert!(!dest.join("archive").exists());
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn atomic_copy_leaves_old_tree_on_failure() {
        let src = scratch("atomic");
        let dest = scratch("atomic-dest");
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("old.txt"), "original").unwrap();

        // Point src at a non-existent path to force an error.
        let bad_src = src.join("does_not_exist");
        let result = copy_dir_atomic(&bad_src, &dest, &[]);
        assert!(result.is_err());
        // Original dest should still be intact.
        assert!(dest.join("old.txt").exists());
        assert_eq!(fs::read_to_string(dest.join("old.txt")).unwrap(), "original");
        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&dest);
    }
}
