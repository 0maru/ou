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
        let matches = fs
            .glob(source_dir, pattern)
            .map_err(|e| OuError::Symlink(format!("glob error for pattern '{pattern}': {e}")))?;

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

fn create_single_symlink(fs: &dyn FileSystem, source: &Path, target: &Path) -> Result<(), OuError> {
    if fs.exists(target) || fs.is_symlink(target) {
        return Ok(());
    }

    if let Some(parent) = target.parent()
        && !fs.exists(parent)
    {
        fs.mkdir_all(parent).map_err(|e| {
            OuError::Symlink(format!(
                "failed to create directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    fs.symlink(source, target).map_err(|e| {
        OuError::Symlink(format!(
            "failed to create symlink {} -> {}: {e}",
            target.display(),
            source.display()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::mock::MockFileSystem;
    use std::path::PathBuf;

    #[test]
    fn test_create_symlinks_literal_file() {
        let fs = MockFileSystem::new().with_file(PathBuf::from("/src/.env"), "SECRET=123");
        let created = create_symlinks(
            &fs,
            Path::new("/src"),
            Path::new("/target"),
            &[".env".to_string()],
        )
        .unwrap();
        assert_eq!(created, vec![".env".to_string()]);
    }

    #[test]
    fn test_create_symlinks_target_exists_skip() {
        // When target already exists, create_single_symlink skips actual symlink creation
        // but create_symlinks still reports the pattern as processed.
        // Verify no actual symlink call is made by checking the mock's symlinks list is empty.
        let fs = MockFileSystem::new()
            .with_file(PathBuf::from("/src/.env"), "SECRET=123")
            .with_file(PathBuf::from("/target/.env"), "ALREADY_EXISTS");
        let _created = create_symlinks(
            &fs,
            Path::new("/src"),
            Path::new("/target"),
            &[".env".to_string()],
        )
        .unwrap();
        // The key behavior: no symlink() call was made because target exists
        assert!(!fs.is_symlink(Path::new("/target/.env")));
    }

    #[test]
    fn test_create_symlinks_no_match() {
        let fs = MockFileSystem::new();
        let created = create_symlinks(
            &fs,
            Path::new("/src"),
            Path::new("/target"),
            &["nonexistent".to_string()],
        )
        .unwrap();
        assert!(created.is_empty());
    }

    #[test]
    fn test_create_symlinks_creates_parent_dirs() {
        let fs = MockFileSystem::new().with_file(PathBuf::from("/src/sub/file.txt"), "content");
        let created = create_symlinks(
            &fs,
            Path::new("/src"),
            Path::new("/target"),
            &["sub/file.txt".to_string()],
        )
        .unwrap();
        assert_eq!(created, vec!["sub/file.txt".to_string()]);
    }
}
