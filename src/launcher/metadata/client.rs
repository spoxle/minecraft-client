use serde::de::DeserializeOwned;

use crate::launcher::Result;

use super::{JavaRuntimeCatalog, RuntimeManifest, VersionManifest, VersionMetadata};

pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const JAVA_RUNTIME_CATALOG_URL: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Clone, Debug)]
pub struct MetadataClient {
    http: reqwest::Client,
}

impl MetadataClient {
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("mc-client/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { http })
    }

    pub fn with_client(http: reqwest::Client) -> Self {
        Self { http }
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub async fn version_manifest(&self) -> Result<VersionManifest> {
        self.get_json(VERSION_MANIFEST_URL).await
    }

    pub async fn version(&self, url: &str) -> Result<VersionMetadata> {
        self.get_json(url).await
    }

    pub async fn java_runtime_catalog(&self) -> Result<JavaRuntimeCatalog> {
        self.get_json(JAVA_RUNTIME_CATALOG_URL).await
    }

    pub async fn runtime_manifest(&self, url: &str) -> Result<RuntimeManifest> {
        self.get_json(url).await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        Ok(self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

impl Default for MetadataClient {
    fn default() -> Self {
        Self::new().expect("the built-in HTTP client configuration is valid")
    }
}
