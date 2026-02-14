use std::path::{Path, PathBuf};

#[allow(dead_code)]
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

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    pub struct MockFileSystem {
        files: Mutex<HashMap<PathBuf, String>>,
        dirs: Mutex<HashSet<PathBuf>>,
        symlinks: Mutex<Vec<(PathBuf, PathBuf)>>,
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: Mutex::new(HashMap::new()),
                dirs: Mutex::new(HashSet::new()),
                symlinks: Mutex::new(Vec::new()),
            }
        }

        pub fn with_file(self, path: impl Into<PathBuf>, content: &str) -> Self {
            self.files
                .lock()
                .unwrap()
                .insert(path.into(), content.to_string());
            self
        }

        pub fn with_dir(self, path: impl Into<PathBuf>) -> Self {
            self.dirs.lock().unwrap().insert(path.into());
            self
        }
    }

    impl FileSystem for MockFileSystem {
        fn symlink(&self, original: &Path, link: &Path) -> Result<(), std::io::Error> {
            self.symlinks
                .lock()
                .unwrap()
                .push((original.to_path_buf(), link.to_path_buf()));
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
                || self.dirs.lock().unwrap().contains(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.dirs.lock().unwrap().contains(path)
        }

        fn is_symlink(&self, path: &Path) -> bool {
            self.symlinks
                .lock()
                .unwrap()
                .iter()
                .any(|(_, link)| link == path)
        }

        fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
        }

        fn write(&self, path: &Path, contents: &str) -> Result<(), std::io::Error> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), contents.to_string());
            Ok(())
        }

        fn mkdir_all(&self, path: &Path) -> Result<(), std::io::Error> {
            self.dirs.lock().unwrap().insert(path.to_path_buf());
            Ok(())
        }

        fn remove_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
            self.dirs.lock().unwrap().remove(path);
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<(), std::io::Error> {
            self.files.lock().unwrap().remove(path);
            Ok(())
        }

        fn glob(&self, dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, std::io::Error> {
            let matcher = globset::Glob::new(pattern)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?
                .compile_matcher();
            let files = self.files.lock().unwrap();
            let mut results = Vec::new();
            for path in files.keys() {
                if let Ok(relative) = path.strip_prefix(dir)
                    && matcher.is_match(relative)
                {
                    results.push(path.clone());
                }
            }
            Ok(results)
        }

        fn canonicalize(&self, path: &Path) -> Result<PathBuf, std::io::Error> {
            Ok(path.to_path_buf())
        }
    }
}
