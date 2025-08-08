use std::path::PathBuf;

use semver::Version;
use serde::Deserialize;
use serde_json::from_str;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    plugin::PluginKind,
    prelude::{LunaticError, Result},
};

#[derive(Deserialize)]
struct ManifestVersionFetcher {
    manifest_version: u32,
}

pub enum Manifest {
    Unstable(UnstableManifest),
}

impl Manifest {
    pub fn get_src(&self) -> &PathBuf {
        match &self {
            Self::Unstable(m) => &m.src,
        }
    }
    pub fn get_kind(&self) -> PluginKind {
        match &self {
            Self::Unstable(m) => m.kind,
        }
    }
}

#[derive(Deserialize)]
struct UnstableManifestPre {
    name: Option<String>,
    kind: Option<PluginKind>,
    rust_ver: Option<Version>,
    display_name: Option<String>,
    src: Option<PathBuf>,
    requirements: Option<Vec<String>>,
    features: Option<Vec<String>>,
}

impl TryFrom<&str> for UnstableManifestPre {
    type Error = serde_json::Error;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        from_str(value)
    }
}

pub struct UnstableManifest {
    pub name: String,
    pub kind: PluginKind,
    pub rust_ver: Option<Version>,
    pub display_name: String,
    pub src: PathBuf,
    pub requirements: Vec<String>,
    pub features: Vec<String>,
}

impl TryFrom<UnstableManifestPre> for UnstableManifest {
    type Error = LunaticError;
    fn try_from(value: UnstableManifestPre) -> std::result::Result<Self, Self::Error> {
        let mut name_clone = None;
        match value.name {
            None => {
                return Err(LunaticError::InvalidManifest {
                    reason:
                        "field \"name\" does not exist. Every plugin requires a canonical name."
                            .to_string(),
                });
            }
            Some(ref s) => {
                if value.display_name.is_none() {
                    name_clone = Some(s.clone())
                }
            }
        }
        match value.src {
            None => {
                return Err(LunaticError::InvalidManifest {
                    reason:
                        "field \"src\" does not exist. Every plugin requires a binary to function."
                            .to_string(),
                });
            }
            Some(ref s) => {
                if !s.exists() {
                    return Err(LunaticError::FileNotFound {
                        path: s.to_path_buf(),
                    });
                }
            }
        }
        if value.kind.is_none() {
            return Err(LunaticError::InvalidManifest { reason: "field \"kind\" does not exist. Plugin manager has no way to know what kind of plugin it is for now.".to_string() });
        }
        // these all SHOULD be some.
        unsafe {
            Ok(UnstableManifest {
                name: value.name.unwrap_unchecked(),
                kind: value.kind.unwrap_unchecked(),
                rust_ver: value.rust_ver,
                display_name: match value.display_name {
                    None => name_clone.unwrap_unchecked(),
                    Some(s) => s,
                },
                src: value.src.unwrap_unchecked(),
                features: value.features.unwrap_or(Vec::new()),
                requirements: value.requirements.unwrap_or(Vec::new()),
            })
        }
    }
}

impl Manifest {
    pub async fn load_manifest(path: PathBuf) -> Result<Manifest> {
        let mut file = File::open(path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents);
        let version: ManifestVersionFetcher = from_str(&contents)?;
        match version.manifest_version {
            0 => Ok(Manifest::Unstable(UnstableManifest::try_from(
                UnstableManifestPre::try_from(contents.as_str())?,
            )?)),
            n => Err(LunaticError::InvalidManifest {
                reason: format!("Unknown manifest version:{n}"),
            }),
        }
    }

    pub fn get_version(&self) -> ManifestVersion {
        match &self {
            Self::Unstable(_) => ManifestVersion::Unstable,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ManifestVersion {
    Unstable,
}
