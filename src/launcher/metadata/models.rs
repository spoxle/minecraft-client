use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::launcher::Platform;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionSummary>,
}

impl VersionManifest {
    pub fn find(&self, id: &str) -> Option<&VersionSummary> {
        self.versions.iter().find(|version| version.id == id)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: VersionType,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMetadata {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: VersionType,
    pub main_class: String,
    pub assets: String,
    pub asset_index: Download,
    pub downloads: VersionDownloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub java_version: Option<JavaRequirement>,
    #[serde(default)]
    pub logging: Option<Logging>,
}

impl VersionMetadata {
    pub fn java_requirement(&self) -> JavaRequirement {
        self.java_version
            .clone()
            .unwrap_or_else(|| JavaRequirement {
                component: "jre-legacy".to_owned(),
                major_version: 8,
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionDownloads {
    pub client: Download,
    #[serde(default)]
    pub client_mappings: Option<Download>,
    #[serde(default)]
    pub server: Option<Download>,
    #[serde(default)]
    pub server_mappings: Option<Download>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    #[serde(default)]
    pub id: Option<String>,
    pub sha1: String,
    pub size: u64,
    #[serde(default)]
    pub total_size: Option<u64>,
    pub url: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRequirement {
    pub component: String,
    pub major_version: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Plain(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub natives: HashMap<String, String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub extract: Option<Extract>,
}

impl Library {
    pub fn is_allowed(&self, platform: Platform) -> bool {
        rules_allow(&self.rules, platform)
    }

    pub fn native_classifier(&self, platform: Platform) -> Option<String> {
        self.natives
            .get(platform.rule_os())
            .map(|value| value.replace("${arch}", platform.native_arch()))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<Download>,
    #[serde(default)]
    pub classifiers: HashMap<String, Download>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: HashMap<String, bool>,
}

impl Rule {
    fn matches(&self, platform: Platform) -> bool {
        let os_matches = self.os.as_ref().is_none_or(|os| {
            os.name
                .as_deref()
                .is_none_or(|name| name == platform.rule_os())
                && os.arch.as_deref().is_none_or(|arch| {
                    arch == platform.rule_arch() || (arch == "x86" && platform.rule_arch() == "x86")
                })
        });
        os_matches && self.features.values().all(|expected| !expected)
    }
}

pub fn rules_allow(rules: &[Rule], platform: Platform) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules.iter().filter(|rule| rule.matches(platform)) {
        allowed = rule.action == RuleAction::Allow;
    }
    allowed
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Logging {
    pub client: ClientLogging,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClientLogging {
    pub argument: String,
    pub file: Download,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub virtual_: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub type JavaRuntimeCatalog = BTreeMap<String, BTreeMap<String, Vec<JavaRuntime>>>;

#[derive(Clone, Debug, Deserialize)]
pub struct JavaRuntime {
    pub availability: Availability,
    pub manifest: RuntimeManifestDownload,
    pub version: RuntimeVersion,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Availability {
    pub group: u32,
    pub progress: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeManifestDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeVersion {
    pub name: String,
    pub released: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeManifest {
    pub files: BTreeMap<String, RuntimeFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeFile {
    #[serde(rename = "type")]
    pub kind: RuntimeFileType,
    #[serde(default)]
    pub downloads: RuntimeDownloads,
    #[serde(default)]
    pub executable: bool,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeFileType {
    File,
    Directory,
    Link,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct RuntimeDownloads {
    #[serde(default)]
    pub raw: Option<RuntimeDownload>,
    #[serde(default)]
    pub lzma: Option<RuntimeDownload>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRecord {
    pub id: String,
    pub name: String,
    pub version_id: String,
    pub created_at_unix_seconds: u64,
    #[serde(default)]
    pub settings: Value,
}

#[cfg(test)]
mod tests {
    use super::{RuntimeManifest, VersionManifest, VersionMetadata};

    #[test]
    fn parses_minimal_mojang_metadata_shapes() {
        let catalog: VersionManifest = serde_json::from_str(r#"{
            "latest":{"release":"1.0","snapshot":"1.1-test"},
            "versions":[{"id":"1.0","type":"release","url":"https://example.test/1.0.json","time":"2026-01-01T00:00:00Z","releaseTime":"2026-01-01T00:00:00Z","sha1":"abc","complianceLevel":1}]
        }"#).unwrap();
        assert_eq!(catalog.find("1.0").unwrap().id, "1.0");

        let version: VersionMetadata = serde_json::from_str(
            r#"{
            "id":"1.0","type":"release","mainClass":"net.minecraft.client.main.Main","assets":"1",
            "assetIndex":{"id":"1","sha1":"abc","size":1,"url":"https://example.test/assets"},
            "downloads":{"client":{"sha1":"def","size":1,"url":"https://example.test/client"}},
            "libraries":[]
        }"#,
        )
        .unwrap();
        assert_eq!(version.java_requirement().major_version, 8);

        let runtime: RuntimeManifest = serde_json::from_str(r#"{
            "files":{"bin/java":{"type":"file","executable":true,"downloads":{"raw":{"sha1":"abc","size":1,"url":"https://example.test/java"}}}}
        }"#).unwrap();
        assert!(runtime.files["bin/java"].executable);
    }
}
