use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::launcher::metadata::{AssetIndex, Download, InstanceRecord, Library, VersionMetadata};
use crate::launcher::{Error, Result};
use crate::task::TaskProgress;
use uuid::Uuid;

use super::download::{DownloadSpec, relative_path};
use super::{Installer, JavaInstall, safe_component};

const ASSET_OBJECT_BASE: &str = "https://resources.download.minecraft.net";
const LIBRARY_BASE: &str = "https://libraries.minecraft.net";

#[derive(Clone, Debug)]
pub struct InstanceInstall {
    pub name: String,
    pub root: PathBuf,
    pub game_directory: PathBuf,
    pub version: VersionMetadata,
    pub java: JavaInstall,
    pub client_jar: PathBuf,
    pub natives_directory: PathBuf,
}

impl Installer {
    pub async fn install_instance(&self, name: &str, version_id: &str) -> Result<InstanceInstall> {
        self.install_instance_with_progress(name, version_id, |_| {})
            .await
    }

    pub async fn install_instance_with_progress<F>(
        &self,
        name: &str,
        version_id: &str,
        progress: F,
    ) -> Result<InstanceInstall>
    where
        F: Fn(TaskProgress),
    {
        safe_component(name)?;
        safe_component(version_id)?;
        progress(TaskProgress::from_count(
            "Checking Minecraft metadata",
            0,
            1,
        ));

        let catalog = self.metadata.version_manifest().await?;
        let summary = catalog
            .find(version_id)
            .ok_or_else(|| Error::MissingMetadata {
                kind: "Minecraft version",
                key: version_id.to_owned(),
                platform: self.platform.runtime_key().to_owned(),
            })?;
        let version_dir = self.paths.versions().join(version_id);
        let version_json = version_dir.join(format!("{version_id}.json"));
        self.downloader
            .one(DownloadSpec {
                url: summary.url.clone(),
                path: version_json.clone(),
                sha1: Some(summary.sha1.clone()),
                size: None,
            })
            .await?;
        let version: VersionMetadata =
            serde_json::from_slice(&tokio::fs::read(version_json).await?)?;
        progress(TaskProgress::from_count(
            "Checking Minecraft metadata",
            1,
            1,
        ));
        let java = self
            .ensure_java_for_with_progress(&version, |percent| {
                progress(TaskProgress {
                    description: "Downloading Java".to_owned(),
                    percent,
                });
            })
            .await?;

        let id = Uuid::new_v4().to_string();
        let root = self.paths.instances().join(&id);
        let game_directory = root.clone();
        let natives_directory = self.paths.natives(&id, version_id);
        tokio::fs::create_dir_all(&game_directory).await?;
        tokio::fs::create_dir_all(&natives_directory).await?;

        let client_jar = version_dir.join(format!("{version_id}.jar"));
        progress(TaskProgress::from_count(
            "Downloading Minecraft client",
            0,
            1,
        ));
        self.downloader
            .one(spec(&version.downloads.client, client_jar.clone()))
            .await?;
        progress(TaskProgress::from_count(
            "Downloading Minecraft client",
            1,
            1,
        ));
        self.install_libraries(&version, &natives_directory, &progress)
            .await?;
        self.install_assets(&version, &game_directory, &progress)
            .await?;
        self.install_logging(&version, &progress).await?;

        progress(TaskProgress::from_count("Finalizing instance", 0, 1));
        let record = InstanceRecord {
            id,
            name: name.to_owned(),
            version_id: version.id.clone(),
            created_at_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            settings: serde_json::Value::Object(Default::default()),
        };
        write_json_atomic(&root.join("instance.json"), &record).await?;
        progress(TaskProgress::from_count("Finalizing instance", 1, 1));

        Ok(InstanceInstall {
            name: name.to_owned(),
            root,
            game_directory,
            version,
            java,
            client_jar,
            natives_directory,
        })
    }

    async fn install_libraries<F>(
        &self,
        version: &VersionMetadata,
        natives: &Path,
        progress: &F,
    ) -> Result<()>
    where
        F: Fn(TaskProgress),
    {
        let libraries_root = self.paths.libraries();
        let mut downloads = Vec::new();
        let mut native_archives = Vec::new();
        let mut seen = HashSet::new();

        for library in version
            .libraries
            .iter()
            .filter(|library| library.is_allowed(self.platform))
        {
            if let Some(artifact) = main_artifact_spec(library, &libraries_root)? {
                push_unique(&mut downloads, &mut seen, artifact);
            }

            if let Some(classifier) = library.native_classifier(self.platform) {
                let archive = if let Some(download) = library
                    .downloads
                    .as_ref()
                    .and_then(|downloads| downloads.classifiers.get(&classifier))
                {
                    spec(
                        download,
                        libraries_root.join(artifact_path(download, library, Some(&classifier))?),
                    )
                } else {
                    derived_library_spec(library, Some(&classifier), &libraries_root)?
                };
                native_archives.push((
                    archive.path.clone(),
                    library
                        .extract
                        .as_ref()
                        .map(|value| value.exclude.clone())
                        .unwrap_or_default(),
                ));
                push_unique(&mut downloads, &mut seen, archive);
            }
        }
        progress(TaskProgress::from_count(
            "Downloading Minecraft libraries",
            0,
            1,
        ));
        self.downloader
            .many_with_progress(downloads, |completed, total| {
                progress(TaskProgress::from_count(
                    "Downloading Minecraft libraries",
                    completed,
                    total,
                ));
            })
            .await?;

        let native_count = native_archives.len();
        progress(TaskProgress::from_count(
            "Preparing native libraries",
            0,
            native_count,
        ));
        for (index, (archive, exclusions)) in native_archives.into_iter().enumerate() {
            let destination = natives.to_owned();
            tokio::task::spawn_blocking(move || {
                extract_native_archive(&archive, &destination, &exclusions)
            })
            .await
            .map_err(|error| Error::Io(std::io::Error::other(error)))??;
            progress(TaskProgress::from_count(
                "Preparing native libraries",
                index + 1,
                native_count,
            ));
        }
        Ok(())
    }

    async fn install_assets<F>(
        &self,
        version: &VersionMetadata,
        game_directory: &Path,
        progress: &F,
    ) -> Result<()>
    where
        F: Fn(TaskProgress),
    {
        let assets = self.paths.assets();
        let index_id = version.asset_index.id.as_deref().unwrap_or(&version.assets);
        safe_component(index_id)?;
        let index_path = assets.join("indexes").join(format!("{index_id}.json"));
        progress(TaskProgress::from_count("Downloading asset index", 0, 1));
        self.downloader
            .one(spec(&version.asset_index, index_path.clone()))
            .await?;
        progress(TaskProgress::from_count("Downloading asset index", 1, 1));
        let index: AssetIndex = serde_json::from_slice(&tokio::fs::read(index_path).await?)?;

        let mut downloads = Vec::with_capacity(index.objects.len());
        for object in index.objects.values() {
            if object.hash.len() < 2 || !object.hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(Error::UnsafePath(object.hash.clone()));
            }
            let prefix = &object.hash[..2];
            downloads.push(DownloadSpec {
                url: format!("{ASSET_OBJECT_BASE}/{prefix}/{}", object.hash),
                path: assets.join("objects").join(prefix).join(&object.hash),
                sha1: Some(object.hash.clone()),
                size: Some(object.size),
            });
        }
        progress(TaskProgress::from_count(
            "Downloading Minecraft assets",
            0,
            1,
        ));
        self.downloader
            .many_with_progress(downloads, |completed, total| {
                progress(TaskProgress::from_count(
                    "Downloading Minecraft assets",
                    completed,
                    total,
                ));
            })
            .await?;

        if index.virtual_ {
            materialize_assets(&assets, &assets.join("virtual").join(index_id), &index).await?;
        }
        if index.map_to_resources {
            materialize_assets(&assets, &game_directory.join("resources"), &index).await?;
        }
        Ok(())
    }

    async fn install_logging<F>(&self, version: &VersionMetadata, progress: &F) -> Result<()>
    where
        F: Fn(TaskProgress),
    {
        let Some(logging) = &version.logging else {
            return Ok(());
        };
        let id = logging
            .client
            .file
            .id
            .as_deref()
            .unwrap_or("client-log-config.xml");
        safe_component(id)?;
        progress(TaskProgress::from_count(
            "Downloading logging configuration",
            0,
            1,
        ));
        self.downloader
            .one(spec(
                &logging.client.file,
                self.paths.assets().join("log_configs").join(id),
            ))
            .await?;
        progress(TaskProgress::from_count(
            "Downloading logging configuration",
            1,
            1,
        ));
        Ok(())
    }
}

fn spec(download: &Download, path: PathBuf) -> DownloadSpec {
    DownloadSpec {
        url: download.url.clone(),
        path,
        sha1: Some(download.sha1.clone()),
        size: Some(download.size),
    }
}

fn main_artifact_spec(library: &Library, root: &Path) -> Result<Option<DownloadSpec>> {
    match &library.downloads {
        Some(downloads) => downloads
            .artifact
            .as_ref()
            .map(|artifact| {
                Ok(spec(
                    artifact,
                    root.join(artifact_path(artifact, library, None)?),
                ))
            })
            .transpose(),
        None => derived_library_spec(library, None, root).map(Some),
    }
}

fn artifact_path(
    download: &Download,
    library: &Library,
    classifier: Option<&str>,
) -> Result<PathBuf> {
    match download.path.as_deref() {
        Some(path) => relative_path(path),
        None => maven_path(&library.name, classifier),
    }
}

fn derived_library_spec(
    library: &Library,
    classifier: Option<&str>,
    root: &Path,
) -> Result<DownloadSpec> {
    let path = maven_path(&library.name, classifier)?;
    let base = library
        .url
        .as_deref()
        .unwrap_or(LIBRARY_BASE)
        .trim_end_matches('/');
    Ok(DownloadSpec {
        url: format!("{base}/{}", path.to_string_lossy().replace('\\', "/")),
        path: root.join(path),
        sha1: None,
        size: None,
    })
}

fn maven_path(coordinate: &str, classifier_override: Option<&str>) -> Result<PathBuf> {
    let (coordinate, extension) = coordinate.split_once('@').unwrap_or((coordinate, "jar"));
    let parts: Vec<_> = coordinate.split(':').collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(Error::UnsafePath(coordinate.to_owned()));
    }
    for part in &parts {
        safe_component(part)?;
    }
    safe_component(extension)?;
    let classifier = classifier_override.or_else(|| parts.get(3).copied());
    let mut file = format!("{}-{}", parts[1], parts[2]);
    if let Some(classifier) = classifier {
        file.push('-');
        file.push_str(classifier);
    }
    file.push('.');
    file.push_str(extension);
    Ok(PathBuf::from(parts[0].replace('.', "/"))
        .join(parts[1])
        .join(parts[2])
        .join(file))
}

fn push_unique(downloads: &mut Vec<DownloadSpec>, seen: &mut HashSet<PathBuf>, spec: DownloadSpec) {
    if seen.insert(spec.path.clone()) {
        downloads.push(spec);
    }
}

fn extract_native_archive(archive: &Path, destination: &Path, exclusions: &[String]) -> Result<()> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        let name_text = name.to_string_lossy();
        if entry.is_dir()
            || exclusions
                .iter()
                .any(|excluded| name_text.starts_with(excluded))
        {
            continue;
        }
        let output = destination.join(name);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(output)?;
        std::io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}

async fn materialize_assets(assets: &Path, destination: &Path, index: &AssetIndex) -> Result<()> {
    for (logical_name, object) in &index.objects {
        let relative = relative_path(logical_name)?;
        let source = assets
            .join("objects")
            .join(&object.hash[..2])
            .join(&object.hash);
        let target = destination.join(relative);
        if target.is_file() {
            continue;
        }
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        if tokio::fs::hard_link(&source, &target).await.is_err() {
            tokio::fs::copy(source, target).await?;
        }
    }
    Ok(())
}

async fn write_json_atomic(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temp = path.with_extension("json.part");
    tokio::fs::write(&temp, serde_json::to_vec_pretty(value)?).await?;
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    tokio::fs::rename(temp, path).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{main_artifact_spec, maven_path};
    use crate::launcher::metadata::Library;
    use std::path::PathBuf;

    #[test]
    fn converts_maven_coordinates_to_paths() {
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3", None).unwrap(),
            PathBuf::from("org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3.jar")
        );
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3", Some("natives-windows")).unwrap(),
            PathBuf::from("org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar")
        );
    }

    #[test]
    fn does_not_invent_an_artifact_for_classifier_only_library_metadata() {
        let library: Library = serde_json::from_str(
            r#"{
                "name":"net.java.jinput:jinput-platform:2.0.5",
                "downloads":{"classifiers":{"natives-windows":{
                    "path":"net/java/jinput/jinput-platform/2.0.5/jinput-platform-2.0.5-natives-windows.jar",
                    "sha1":"abc","size":1,"url":"https://example.test/native.jar"
                }}}
            }"#,
        )
        .unwrap();

        assert!(
            main_artifact_spec(&library, &PathBuf::from("libraries"))
                .unwrap()
                .is_none()
        );
    }
}
