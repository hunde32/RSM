use crate::error::RsmError;
use glob::Pattern;
use std::fs;
use std::os::unix::fs::symlink as unix_symlink;
use std::path::{Path, PathBuf};
use tracing::debug;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct SyncTask {
    pub source: PathBuf,
    pub target: PathBuf,
}

pub fn expand_tilde(path: &Path) -> PathBuf {
    if !path.starts_with("~") {
        return path.to_path_buf();
    }
    let mut new_path = directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_default();

    if path.components().count() > 1 {
        new_path.push(path.strip_prefix("~").unwrap());
    }
    new_path
}

pub fn resolve_tasks(
    source: &Path,
    target: &Path,
    recursive: bool,
    global_ignores: &[String],
    local_ignores: &[String],
) -> Result<Vec<SyncTask>, RsmError> {
    let exp_source = expand_tilde(source);
    let exp_target = expand_tilde(target);

    if !exp_source.exists() {
        return Err(RsmError::SourceMissing(exp_source));
    }

    let ignores: Vec<String> = global_ignores
        .iter()
        .chain(local_ignores.iter())
        .cloned()
        .collect();

    let mut tasks = Vec::new();

    if recursive && exp_source.is_dir() {
        let walker = WalkDir::new(&exp_source).into_iter();
        for entry in walker.filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignores.iter().any(|ig| {
                if name == *ig {
                    return true;
                }
                if let Ok(pat) = Pattern::new(ig) {
                    if pat.matches(&name) {
                        return true;
                    }
                }
                false
            })
        }) {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let rel_path = path.strip_prefix(&exp_source).unwrap();
                    tasks.push(SyncTask {
                        source: path.to_path_buf(),
                        target: exp_target.join(rel_path),
                    });
                }
            }
        }
    } else {
        let name = exp_source.file_name().unwrap().to_string_lossy();
        let is_ignored = ignores.iter().any(|ig| {
            if name == *ig {
                return true;
            }
            if let Ok(pat) = Pattern::new(ig) {
                if pat.matches(&name) {
                    return true;
                }
            }
            false
        });

        if !is_ignored {
            tasks.push(SyncTask {
                source: exp_source,
                target: exp_target,
            });
        }
    }

    Ok(tasks)
}

pub fn prune_dead_links(
    target_root: &Path,
    source_root: &Path,
    dry_run: bool,
) -> Result<(), RsmError> {
    let exp_target = expand_tilde(target_root);
    let exp_source = expand_tilde(source_root);

    if !exp_target.exists() || !exp_target.is_dir() {
        return Ok(());
    }

    for entry in WalkDir::new(&exp_target).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_symlink() {
            if let Ok(linked_to) = fs::read_link(path) {
                if linked_to.starts_with(&exp_source) && !linked_to.exists() {
                    debug!("Pruning dead link: {}", path.display());
                    if !dry_run {
                        fs::remove_file(path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn create_link(task: &SyncTask, force: bool, dry_run: bool) -> Result<(), RsmError> {
    if task.target.exists() || task.target.is_symlink() {
        if force {
            if !dry_run {
                if task.target.is_dir() && !task.target.is_symlink() {
                    fs::remove_dir_all(&task.target)?;
                } else {
                    fs::remove_file(&task.target)?;
                }
            }
        } else {
            return Err(RsmError::TargetExists(task.target.clone()));
        }
    }

    if let Some(parent) = task.target.parent() {
        if !parent.exists() && !dry_run {
            fs::create_dir_all(parent)?;
        }
    }

    if !dry_run {
        let abs_source = fs::canonicalize(&task.source)
            .map_err(|_| RsmError::PathResolution(task.source.clone()))?;
        unix_symlink(abs_source, &task.target)?;
    }

    Ok(())
}
