use std::path::Path;

use orion_conf::{
    TomlIO,
    error::{ConfIOReason, OrionConfResult},
};
use orion_error::StructError;

use crate::config::backup_clean;

use super::ConfigLifecycle;

impl<T> ConfigLifecycle for T
where
    T: TomlIO<T> + serde::Serialize + serde::de::DeserializeOwned,
{
    fn load(path: &str) -> OrionConfResult<Self>
    where
        Self: Sized,
    {
        T::load_toml(Path::new(path))
    }

    fn init(&self, path: &str) -> OrionConfResult<()>
    where
        Self: Sized,
    {
        Self::safe_clean(path)?;
        self.save(path)
    }

    fn safe_clean(path: &str) -> OrionConfResult<()> {
        backup_clean(path).map_err(|err| StructError::from(ConfIOReason::Other(err.to_string())))
    }

    fn save(&self, path: &str) -> OrionConfResult<()> {
        self.save_toml(Path::new(path))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ConfigLifecycle;
    use serde_derive::{Deserialize, Serialize};
    use tempfile::TempDir;

    /// `orion_conf` blanket-impls `TomlIO<T> for T`, so any serde struct picks up
    /// the `ConfigLifecycle` impl above.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestConf {
        name: String,
        count: u32,
    }

    fn sample() -> TestConf {
        TestConf {
            name: "orion".to_string(),
            count: 3,
        }
    }

    #[test]
    fn save_then_load_round_trips() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");

        sample().save(path_str).expect("save");

        assert_eq!(TestConf::load(path_str).expect("load"), sample());
        assert!(
            std::fs::read_to_string(&path)
                .expect("read")
                .contains("orion")
        );
    }

    #[test]
    fn init_creates_a_fresh_file() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");

        sample().init(path_str).expect("init");

        assert_eq!(TestConf::load(path_str).expect("load"), sample());
    }

    #[test]
    fn init_backs_up_an_existing_file_before_overwriting() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "name = \"stale\"\ncount = 1\n").expect("seed file");

        sample().init(path_str).expect("init");

        let backup = format!("{path_str}.bak");
        assert!(
            std::fs::read_to_string(&backup)
                .expect("read backup")
                .contains("stale"),
            "init should back up the previous content"
        );
        assert_eq!(TestConf::load(path_str).expect("load"), sample());
    }

    #[test]
    fn try_load_returns_none_when_the_file_is_missing() {
        let tmp = TempDir::new().expect("tempdir");
        let path_str = tmp
            .path()
            .join("absent.toml")
            .to_str()
            .expect("utf8 path")
            .to_string();

        assert_eq!(TestConf::try_load(&path_str), None);
    }

    #[test]
    fn try_load_returns_the_config_when_readable() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        sample().save(path_str).expect("save");

        assert_eq!(TestConf::try_load(path_str), Some(sample()));
    }

    #[test]
    fn try_load_swallows_broken_content() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("conf.toml");
        let path_str = path.to_str().expect("utf8 path");
        std::fs::write(&path, "= = = not a toml document").expect("seed file");

        assert_eq!(TestConf::try_load(path_str), None);
    }
}
