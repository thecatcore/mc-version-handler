use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct VersionFile {
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Version {
    pub name: String,
    pub url: String,
    #[serde(rename = "oldJava")]
    pub old_java: bool,
}

pub fn parse_version_manifest(version_str: &str) -> serde_json::Result<VersionFile> {
    serde_json::from_str(version_str)
}
