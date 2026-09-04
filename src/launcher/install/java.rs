use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::from_slice;

use crate::launcher::metadata::{RuntimeFileType, RuntimeManifest, VersionMetadata};
use crate::launcher::{Error, Result};
use crate::task::TaskProgress;

use super::download::{DownloadSpec, relative_path};
use super::{Installer, safe_component};

#[derive(Clone, Debug)]
pub struct JavaInstall {
    pub component: String,
    pub major_version: u32,
    pub runtime_version: String,
    pub root: PathBuf,
    pub executable: PathBuf,
}

impl Installer {
    pub async fn ensure_java_for(&self, version: &VersionMetadata) -> Result<JavaInstall> {
        self.ensure_java_for_with_progress(version, |_| {}).await
    }

    pub(crate) async fn ensure_java_for_with_progress<F>(
        &self,
        version: &VersionMetadata,
        progress: F,
    ) -> Result<JavaInstall>
    where
        F: Fn(u8),
    {
        progress(0);
        let requirement = version.java_requirement();
        safe_component(&requirement.component)?;
        let catalog = self.metadata.java_runtime_catalog().await?;
        let runtimes = catalog
            .get(self.platform.runtime_key())
            .and_then(|components| components.get(&requirement.component))
            .ok_or_else(|| Error::MissingMetadata {
                kind: "Java component",
                key: requirement.component.clone(),
                platform: self.platform.runtime_key().to_owned(),
            })?;
        let runtime = runtimes
            .iter()
            .filter(|runtime| runtime.availability.progress == 100)
            .max_by_key(|runtime| &runtime.version.released)
            .ok_or_else(|| Error::MissingMetadata {
                kind: "available Java runtime",
                key: requirement.component.clone(),
                platform: self.platform.runtime_key().to_owned(),
            })?;
        safe_component(&runtime.version.name)?;

        let root = self.paths.runtime(
            &requirement.component,
            self.platform.runtime_key(),
            &runtime.version.name,
        );
        let marker = root.join(".manifest-sha1");
        if tokio::fs::read_to_string(&marker).await.ok().as_deref() == Some(&runtime.manifest.sha1)
            && let Some(executable) = java_executable(&root).await
        {
            progress(100);
            return Ok(JavaInstall {
                component: requirement.component,
                major_version: requirement.major_version,
                runtime_version: runtime.version.name.clone(),
                root,
                executable,
            });
        }

        let manifest_path = self
            .paths
            .cache()
            .join("metadata/runtime-manifests")
            .join(format!("{}.json", runtime.manifest.sha1));
        self.downloader
            .one(DownloadSpec {
                url: runtime.manifest.url.clone(),
                path: manifest_path.clone(),
                sha1: Some(runtime.manifest.sha1.clone()),
                size: Some(runtime.manifest.size),
            })
            .await?;
        let manifest: RuntimeManifest = from_slice(&tokio::fs::read(manifest_path).await?)?;
        install_runtime_files(self, &root, &manifest, &progress).await?;

        tokio::fs::write(&marker, &runtime.manifest.sha1).await?;
        let executable = java_executable(&root)
            .await
            .ok_or_else(|| Error::MissingMetadata {
                kind: "Java executable",
                key: requirement.component.clone(),
                platform: self.platform.runtime_key().to_owned(),
            })?;
        progress(100);
        Ok(JavaInstall {
            component: requirement.component,
            major_version: requirement.major_version,
            runtime_version: runtime.version.name.clone(),
            root,
            executable,
        })
    }
}

async fn install_runtime_files(
    installer: &Installer,
    root: &Path,
    manifest: &RuntimeManifest,
    progress: &impl Fn(u8),
) -> Result<()> {
    tokio::fs::create_dir_all(root).await?;
    for (name, file) in &manifest.files {
        if file.kind == RuntimeFileType::Directory {
            tokio::fs::create_dir_all(root.join(relative_path(name)?)).await?;
        }
    }

    let downloads = manifest
        .files
        .iter()
        .filter_map(|(name, file)| {
            let download = file.downloads.raw.as_ref()?;
            Some(relative_path(name).map(|path| DownloadSpec {
                url: download.url.clone(),
                path: root.join(path),
                sha1: Some(download.sha1.clone()),
                size: Some(download.size),
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    installer
        .downloader
        .many_with_progress(downloads, |completed, total| {
            let downloaded = TaskProgress::from_count("", completed, total).percent;
            progress(((u16::from(downloaded) * 95) / 100) as u8);
        })
        .await?;

    for (name, file) in &manifest.files {
        let path = root.join(relative_path(name)?);
        if file.executable && path.is_file() {
            make_executable(&path)?;
        }
        if file.kind == RuntimeFileType::Link && !path.exists() {
            let target = file
                .target
                .as_deref()
                .ok_or_else(|| Error::UnsafePath(name.clone()))?;
            create_link(target, &path)?;
        }
    }
    Ok(())
}

async fn java_executable(root: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let candidates = [root.join("bin/javaw.exe"), root.join("bin/java.exe")];
    #[cfg(target_os = "macos")]
    let candidates = [
        root.join("jre.bundle/Contents/Home/bin/java"),
        root.join("bin/java"),
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    let candidates = [root.join("bin/java"), root.join("jre/bin/java")];
    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn create_link(target: &str, path: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, path)?;
    Ok(())
}

#[cfg(windows)]
fn create_link(target: &str, path: &Path) -> Result<()> {
    std::os::windows::fs::symlink_file(target, path)?;
    Ok(())
}
