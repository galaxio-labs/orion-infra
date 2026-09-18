use derive_more::From;
use orion_error::OrionError;
use orion_error::conversion::{ConvErr, ToStructError};
use orion_error::prelude::{ErrorWith, SourceErr};
use orion_error::{StructError, UnifiedReason};
use std::{fs, path::Path};

#[derive(Clone, Debug, PartialEq, From, OrionError)]
pub enum PathReason {
    #[orion_error(identity = "biz.path.brief")]
    Brief(String),
    #[orion_error(transparent)]
    Unified(UnifiedReason),
}

pub type PathResult<T> = Result<T, StructError<PathReason>>;
pub type PathError = StructError<PathReason>;

pub fn make_clean_path(path: &Path) -> PathResult<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .source_err(PathReason::system_error(), "remove_dir_all")
            .with_context(path)?;
    }
    std::fs::create_dir_all(path)
        .source_err(UnifiedReason::system_error(), "create_dir_all")
        .conv_err()
        .with_context(path)?;
    Ok(())
}

pub fn ensure_path<P: AsRef<Path>>(path: P) -> PathResult<P> {
    if !path.as_ref().exists() {
        std::fs::create_dir_all(path.as_ref())
            .source_err(PathReason::system_error(), "create_dir_all")
            .with_context(path.as_ref())?;
    }
    Ok(path)
}

pub fn make_new_path(path: &Path) -> PathResult<()> {
    if path.exists() {
        return PathReason::from(UnifiedReason::resource_error())
            .err_result()
            .doing("path exists")
            .with_context(path);
    }
    std::fs::create_dir_all(path)
        .source_err(UnifiedReason::system_error(), "create_dir_all")
        .conv_err()?;
    Ok(())
}

pub fn get_sub_dirs(path: &Path) -> PathResult<Vec<std::path::PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(path)
        .source_err(PathReason::resource_error(), "read_dir")
        .with_context(path)
        .doing("read sub dirs")?
    {
        let entry = entry.source_err(PathReason::resource_error(), "read_dir_entry")?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn dir_names(paths: &[std::path::PathBuf]) -> Vec<String> {
        let mut names: Vec<String> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn ensure_path_creates_missing_dir_and_returns_input() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("a").join("b");
        assert!(!target.exists());

        let out = ensure_path(&target).expect("ensure_path should succeed");

        assert_eq!(out, target.as_path());
        assert!(target.is_dir());
    }

    #[test]
    fn ensure_path_keeps_existing_dir_untouched() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("exists");
        std::fs::create_dir(&target).expect("create_dir");
        let marker = target.join("keep.txt");
        std::fs::write(&marker, "x").expect("write marker");

        assert!(ensure_path(&target).is_ok());

        assert!(marker.exists(), "ensure_path must not wipe an existing dir");
    }

    #[test]
    fn make_new_path_creates_when_absent() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("fresh").join("deep");

        assert!(make_new_path(&target).is_ok());

        assert!(target.is_dir());
    }

    #[test]
    fn make_new_path_rejects_existing_path() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("taken");
        std::fs::create_dir(&target).expect("create_dir");

        assert!(make_new_path(&target).is_err());
    }

    #[test]
    fn make_clean_path_empties_existing_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("dirty");
        std::fs::create_dir(&target).expect("create_dir");
        std::fs::write(target.join("stale.txt"), "old").expect("write stale");

        assert!(make_clean_path(&target).is_ok());

        assert!(target.is_dir());
        assert_eq!(std::fs::read_dir(&target).expect("read_dir").count(), 0);
    }

    #[test]
    fn make_clean_path_creates_when_absent() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("nope").join("deep");

        assert!(make_clean_path(&target).is_ok());

        assert!(target.is_dir());
    }

    #[test]
    fn get_sub_dirs_returns_directories_only() {
        let tmp = TempDir::new().expect("tempdir");
        std::fs::create_dir(tmp.path().join("d1")).expect("create d1");
        std::fs::create_dir(tmp.path().join("d2")).expect("create d2");
        std::fs::write(tmp.path().join("f1.txt"), "x").expect("write file");

        let dirs = get_sub_dirs(tmp.path()).expect("get_sub_dirs should succeed");

        assert_eq!(dir_names(&dirs), vec!["d1".to_string(), "d2".to_string()]);
    }

    #[test]
    fn get_sub_dirs_fails_on_missing_path() {
        let tmp = TempDir::new().expect("tempdir");
        let missing = tmp.path().join("missing");

        assert!(get_sub_dirs(&missing).is_err());
    }
}
