use std::{
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use futures_util::{StreamExt, stream};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::launcher::{Error, Result};

const DOWNLOAD_CONCURRENCY: usize = 16;

#[derive(Clone, Debug)]
pub(super) struct DownloadSpec {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, Debug)]
pub(super) struct Downloader {
    http: reqwest::Client,
    sequence: Arc<AtomicU64>,
}

impl Downloader {
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn many_with_progress<F>(
        &self,
        downloads: Vec<DownloadSpec>,
        progress: F,
    ) -> Result<()>
    where
        F: Fn(usize, usize),
    {
        let total = downloads.len();
        if total == 0 {
            progress(0, 0);
            return Ok(());
        }
        let mut pending = stream::iter(
            downloads
                .into_iter()
                .map(|spec| async move { self.one(spec).await }),
        )
        .buffer_unordered(DOWNLOAD_CONCURRENCY);
        let mut completed = 0;
        while let Some(result) = pending.next().await {
            result?;
            completed += 1;
            progress(completed, total);
        }
        Ok(())
    }

    pub async fn one(&self, spec: DownloadSpec) -> Result<()> {
        if spec.path.is_file() && verify(&spec.path, spec.sha1.as_deref(), spec.size).await? {
            return Ok(());
        }
        if let Some(parent) = spec.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let suffix = self.sequence.fetch_add(1, Ordering::Relaxed);
        let temp = spec
            .path
            .with_extension(format!("part-{}-{suffix}", std::process::id()));
        let response = self.http.get(&spec.url).send().await?.error_for_status()?;
        let mut stream = response.bytes_stream();
        let mut file = tokio::fs::File::create(&temp).await?;
        let mut hasher = Sha1::new();
        let mut size = 0_u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            size += chunk.len() as u64;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);

        let actual_hash = format!("{:x}", hasher.finalize());
        if spec.size.is_some_and(|expected| expected != size) {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(Error::Integrity {
                path: spec.path,
                expected: spec.size.unwrap().to_string(),
                actual: size.to_string(),
            });
        }
        if spec
            .sha1
            .as_ref()
            .is_some_and(|expected| !expected.eq_ignore_ascii_case(&actual_hash))
        {
            let _ = tokio::fs::remove_file(&temp).await;
            return Err(Error::Integrity {
                path: spec.path,
                expected: spec.sha1.unwrap(),
                actual: actual_hash,
            });
        }

        if spec.path.exists() {
            tokio::fs::remove_file(&spec.path).await?;
        }
        tokio::fs::rename(temp, spec.path).await?;
        Ok(())
    }
}

pub(super) fn relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(Error::UnsafePath(value.to_owned()));
    }
    Ok(path.to_owned())
}

async fn verify(
    path: &Path,
    expected_sha1: Option<&str>,
    expected_size: Option<u64>,
) -> Result<bool> {
    if let Some(size) = expected_size
        && tokio::fs::metadata(path).await?.len() != size
    {
        return Ok(false);
    }
    let Some(expected_sha1) = expected_sha1 else {
        return Ok(true);
    };
    let mut file = tokio::fs::File::open(path).await?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut hasher = Sha1::new();
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(expected_sha1.eq_ignore_ascii_case(&format!("{:x}", hasher.finalize())))
}

#[cfg(test)]
mod tests {
    use super::relative_path;

    #[test]
    fn metadata_paths_must_remain_relative() {
        assert!(relative_path("com/mojang/example.jar").is_ok());
        assert!(relative_path("../outside").is_err());
        assert!(relative_path("folder/../outside").is_err());
        assert!(relative_path("/absolute").is_err());
    }
}
