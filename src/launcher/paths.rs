use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use super::{Error, Result};

#[derive(Clone, Debug)]
pub struct LauncherPaths {
    data: PathBuf,
    cache: PathBuf,
}

impl LauncherPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "Spoxle", "mc-client").ok_or(Error::DataDirectory)?;
        Ok(Self {
            data: dirs.data_local_dir().to_owned(),
            cache: dirs.cache_dir().to_owned(),
        })
    }

    pub fn from_roots(data: impl Into<PathBuf>, cache: impl Into<PathBuf>) -> Self {
        Self {
            data: data.into(),
            cache: cache.into(),
        }
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }

    pub fn instances(&self) -> PathBuf {
        self.data.join("instances")
    }

    pub fn runtime(&self, component: &str, platform: &str, version: &str) -> PathBuf {
        self.data
            .join("runtimes")
            .join(component)
            .join(platform)
            .join(version)
    }

    pub fn libraries(&self) -> PathBuf {
        self.data.join("minecraft").join("libraries")
    }

    pub fn assets(&self) -> PathBuf {
        self.data.join("minecraft").join("assets")
    }

    pub fn versions(&self) -> PathBuf {
        self.data.join("minecraft").join("versions")
    }

    pub fn natives(&self, instance_id: &str, version_id: &str) -> PathBuf {
        self.cache
            .join("natives")
            .join(instance_id)
            .join(version_id)
    }
}
