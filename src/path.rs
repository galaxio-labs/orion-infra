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
