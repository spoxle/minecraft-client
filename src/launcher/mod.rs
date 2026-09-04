pub mod error;
pub mod install;
pub mod metadata;
pub mod paths;
pub mod platform;

pub use error::{Error, Result};
pub use install::{Installer, InstanceInstall, JavaInstall};
pub use metadata::{MetadataClient, VersionManifest, VersionMetadata};
pub use paths::LauncherPaths;
pub use platform::Platform;
