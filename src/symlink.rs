use std::path::Path;

use crate::error::OuError;
use crate::fs::FileSystem;

pub fn create_symlinks(
    fs: &dyn FileSystem,
    source_dir: &Path,
    target_dir: &Path,
    patterns: &[String],
) -> Result<Vec<String>, OuError> {
    let mut created = Vec::new();

    for pattern in patterns {
        let matches = fs.glob(source_dir, pattern).map_err(|e| {
            OuError::Symlink(format!("glob error for pattern '{pattern}': {e}"))
        })?;

        if matches.is_empty() {
            let source_path = source_dir.join(pattern);
            if fs.exists(&source_path) {
                let target_path = target_dir.join(pattern);
                create_single_symlink(fs, &source_path, &target_path)?;
                created.push(pattern.clone());
            }
            continue;
        }

        for source_path in matches {
            let relative = source_path
                .strip_prefix(source_dir)
                .map_err(|e| OuError::Symlink(format!("path error: {e}")))?;
            let target_path = target_dir.join(relative);
            create_single_symlink(fs, &source_path, &target_path)?;
            created.push(relative.to_string_lossy().to_string());
        }
    }

    Ok(created)
}

fn create_single_symlink(
    fs: &dyn FileSystem,
    source: &Path,
    target: &Path,
) -> Result<(), OuError> {
    if fs.exists(target) || fs.is_symlink(target) {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        if !fs.exists(parent) {
            fs.mkdir_all(parent).map_err(|e| {
                OuError::Symlink(format!("failed to create directory {}: {e}", parent.display()))
            })?;
        }
    }

    fs.symlink(source, target).map_err(|e| {
        OuError::Symlink(format!(
            "failed to create symlink {} -> {}: {e}",
            target.display(),
            source.display()
        ))
    })
}
