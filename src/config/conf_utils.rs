use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::types::AnyResult;
use anyhow::{Context, anyhow};

fn handle_exists_file(path: &str, cover: bool) -> AnyResult<()> {
    if std::path::Path::new(path).exists() {
        if cover {
            std::fs::remove_file(path)?;
        } else {
            return Err(anyhow!("{} already exists", path));
        }
    }
    Ok(())
}

pub fn save_data(data: &str, path_str: &str, cover: bool) -> AnyResult<()> {
    handle_exists_file(path_str, cover)?;
    let path = Path::new(path_str);
    if let Some(value) = path.parent() {
        std::fs::create_dir_all(value)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(data.as_bytes())?;
    println!("success init : {path_str} ");
    Ok(())
}

pub fn backup_clean(path: &str) -> AnyResult<()> {
    if std::path::Path::new(path).exists() {
        std::fs::copy(path, format!("{path}.bak"))?;
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn clear_file(path: &str) {
    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("clean {path} failed!"));
    }
}

pub fn read_file(path: &str) -> AnyResult<String> {
    let mut f = File::open(path).with_context(|| format!("conf file not found: {path}"))?;
    let mut buffer = Vec::with_capacity(10240);
    f.read_to_end(&mut buffer)
        .with_context(|| format!("conf file: {path}"))?;
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn save_data_creates_missing_parent_dirs() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("nested").join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");

        save_data("key = 1", path_str, false).expect("save_data");

        assert_eq!(read_file(path_str).expect("read_file"), "key = 1");
    }

    #[test]
    fn save_data_refuses_to_overwrite_without_cover() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "old").expect("seed file");

        let err = save_data("new", path_str, false).expect_err("must refuse");

        assert!(err.to_string().contains("already exists"), "got: {err}");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "old");
    }

    #[test]
    fn save_data_overwrites_when_cover_is_set() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "old").expect("seed file");

        save_data("new", path_str, true).expect("save_data with cover");

        assert_eq!(std::fs::read_to_string(&path).expect("read"), "new");
    }

    #[test]
    fn read_file_reports_a_missing_file() {
        let tmp = TempDir::new().expect("tempdir");
        let path_str = tmp
            .path()
            .join("absent.toml")
            .to_str()
            .expect("utf8 path")
            .to_string();

        let err = read_file(&path_str).expect_err("must fail");

        assert!(
            err.to_string().contains("conf file not found"),
            "got: {err}"
        );
    }

    #[test]
    fn backup_clean_moves_the_original_aside() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "content").expect("seed file");

        backup_clean(path_str).expect("backup_clean");

        assert!(!path.exists());
        assert_eq!(
            std::fs::read_to_string(format!("{path_str}.bak")).expect("read bak"),
            "content"
        );
    }

    #[test]
    fn backup_clean_is_a_noop_when_the_file_is_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("absent.toml");
        let path_str = path.to_str().expect("utf8 path");

        backup_clean(path_str).expect("backup_clean");

        assert!(!path.exists());
        assert!(!std::path::Path::new(&format!("{path_str}.bak")).exists());
    }

    #[test]
    fn clear_file_removes_the_file_and_tolerates_a_missing_one() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "x").expect("seed file");

        clear_file(path_str);
        assert!(!path.exists());

        clear_file(path_str);
    }
}
