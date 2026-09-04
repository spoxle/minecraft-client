mod download;
mod instance;
mod java;

pub use instance::InstanceInstall;
pub use java::JavaInstall;

use crate::launcher::{LauncherPaths, MetadataClient, Platform, Result};

use download::Downloader;

#[derive(Clone, Debug)]
pub struct Installer {
    metadata: MetadataClient,
    paths: LauncherPaths,
    platform: Platform,
    downloader: Downloader,
}

impl Installer {
    pub fn new(paths: LauncherPaths) -> Result<Self> {
        let metadata = MetadataClient::new()?;
        let platform = Platform::current()?;
        let downloader = Downloader::new(metadata.http().clone());
        Ok(Self {
            metadata,
            paths,
            platform,
            downloader,
        })
    }

    pub fn discover() -> Result<Self> {
        Self::new(LauncherPaths::discover()?)
    }

    pub fn metadata(&self) -> &MetadataClient {
        &self.metadata
    }
    pub fn paths(&self) -> &LauncherPaths {
        &self.paths
    }
    pub fn platform(&self) -> Platform {
        self.platform
    }
}

pub(crate) fn safe_component(value: &str) -> Result<&str> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains(['/', '\\'])
        || value.chars().any(|character| character.is_control())
    {
        return Err(crate::launcher::Error::UnsafePath(value.to_owned()));
    }
    Ok(value)
}
