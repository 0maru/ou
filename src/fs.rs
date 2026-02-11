use std::path::{Path, PathBuf};

pub trait FileSystem: Send + Sync {
    fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error>;
    fn exists(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_symlink(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error>;
    fn write(&self, path: &Path, contents: &str) -> Result<(), std::io::Error>;
    fn mkdir_all(&self, path: &Path) -> Result<(), std::io::Error>;
    fn remove_dir_all(&self, path: &Path) -> Result<(), std::io::Error>;
    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error>;
    fn glob(&self, dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, std::io::Error>;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error>;
}

pub struct OsFileSystem;

impl FileSystem for OsFileSystem {
    fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(original, link)
        }
        #[cfg(windows)]
        {
            if original.is_dir() {
                std::os::windows::fs::symlink_dir(original, link)
            } else {
                std::os::windows::fs::symlink_file(original, link)
            }
        }
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_symlink(&self, path: &Path) -> bool {
        path.symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
    }

    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }

    fn write(&self, path: &Path, contents: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, contents)
    }

    fn mkdir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(path)
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::remove_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::remove_file(path)
    }

    fn glob(&self, dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
        let matcher = globset::Glob::new(pattern)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
            .compile_matcher();

        let mut results = Vec::new();
        collect_glob_matches(dir, dir, &matcher, &mut results)?;
        Ok(results)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
        std::fs::canonicalize(path)
    }
}

fn collect_glob_matches(
    base: &Path,
    dir: &Path,
    matcher: &globset::GlobMatcher,
    results: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        if matcher.is_match(relative) {
            results.push(path.clone());
        }
        if path.is_dir() && !path.is_symlink() {
            collect_glob_matches(base, &path, matcher, results)?;
        }
    }
    Ok(())
}
