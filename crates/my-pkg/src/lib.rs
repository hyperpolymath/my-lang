// SPDX-License-Identifier: MIT
//! Package Manager for My Language
//!
//! Provides dependency management and project tooling:
//! - Project creation and initialization
//! - Dependency resolution
//! - Package registry integration
//! - Build orchestration

use petgraph::graph::DiGraph;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Package manager errors
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("manifest not found: {0}")]
    ManifestNotFound(PathBuf),

    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("dependency conflict: {name} requires {required}, but {found} is installed")]
    DependencyConflict {
        name: String,
        required: String,
        found: String,
    },

    #[error("package not found: {0}")]
    PackageNotFound(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Project manifest (my.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub ai: AIConfig,
    #[serde(default)]
    pub dialects: DialectConfig,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

fn default_edition() -> String {
    "2024".to_string()
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}

/// Detailed dependency with features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    pub version: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
}

/// AI configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AIConfig {
    #[serde(default, rename = "default-model")]
    pub default_model: Option<String>,
    #[serde(default)]
    pub cache: bool,
}

/// Dialect configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialectConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
}

/// Lock file (my.lock)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub packages: Vec<LockedPackage>,
}

/// Locked package version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub checksum: Option<String>,
    pub source: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Dependency resolver
pub struct Resolver {
    registry: Registry,
}

impl Resolver {
    pub fn new(registry: Registry) -> Self {
        Resolver { registry }
    }

    /// Resolve dependencies from manifest
    pub async fn resolve(&self, manifest: &Manifest) -> Result<LockFile, PkgError> {
        let mut graph: DiGraph<(String, Version), ()> = DiGraph::new();
        let mut resolved = HashMap::new();

        // Add root package
        let root_version = Version::parse(&manifest.package.version)
            .map_err(|e| PkgError::InvalidManifest(e.to_string()))?;

        // Resolve each dependency
        for (name, dep) in &manifest.dependencies {
            self.resolve_dependency(name, dep, &mut graph, &mut resolved)
                .await?;
        }

        // Convert to lock file
        let packages: Vec<LockedPackage> = resolved
            .into_iter()
            .map(|(name, version)| LockedPackage {
                name,
                version: version.to_string(),
                checksum: None,
                source: "registry".to_string(),
                dependencies: vec![],
            })
            .collect();

        Ok(LockFile { packages })
    }

    async fn resolve_dependency(
        &self,
        name: &str,
        dep: &Dependency,
        graph: &mut DiGraph<(String, Version), ()>,
        resolved: &mut HashMap<String, Version>,
    ) -> Result<(), PkgError> {
        let version_req = match dep {
            Dependency::Simple(v) => VersionReq::parse(v)
                .map_err(|e| PkgError::InvalidManifest(e.to_string()))?,
            Dependency::Detailed(d) => {
                if let Some(v) = &d.version {
                    VersionReq::parse(v)
                        .map_err(|e| PkgError::InvalidManifest(e.to_string()))?
                } else {
                    VersionReq::STAR
                }
            }
        };

        // TODO: Query registry for available versions
        // For now, just use the latest version that matches
        let version = Version::new(0, 1, 0); // Placeholder

        if version_req.matches(&version) {
            resolved.insert(name.to_string(), version);
        } else {
            return Err(PkgError::DependencyConflict {
                name: name.to_string(),
                required: version_req.to_string(),
                found: version.to_string(),
            });
        }

        Ok(())
    }
}

/// Package registry client
pub struct Registry {
    base_url: String,
    client: reqwest::Client,
}

impl Registry {
    pub fn new(base_url: &str) -> Self {
        Registry {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn default_registry() -> Self {
        Registry::new("https://registry.my-lang.dev")
    }

    /// Fetch package metadata
    pub async fn fetch_package(&self, name: &str) -> Result<PackageMetadata, PkgError> {
        let url = format!("{}/api/v1/packages/{}", self.base_url, name);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| PkgError::NetworkError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PkgError::PackageNotFound(name.to_string()));
        }

        response
            .json()
            .await
            .map_err(|e| PkgError::NetworkError(e.to_string()))
    }

    /// Publish a package
    pub async fn publish(&self, tarball: &[u8], token: &str) -> Result<(), PkgError> {
        let url = format!("{}/api/v1/publish", self.base_url);

        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .body(tarball.to_vec())
            .send()
            .await
            .map_err(|e| PkgError::NetworkError(e.to_string()))?;

        Ok(())
    }
}

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub versions: Vec<VersionMetadata>,
}

/// Version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub version: String,
    pub checksum: String,
    pub yanked: bool,
}

/// Package cache
pub struct PackageCache {
    cache_dir: PathBuf,
}

impl PackageCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        PackageCache { cache_dir }
    }

    pub fn default_cache() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PackageCache::new(PathBuf::from(home).join(".my").join("cache"))
    }

    /// Get cached package path
    pub fn get(&self, name: &str, version: &Version) -> Option<PathBuf> {
        let path = self
            .cache_dir
            .join("packages")
            .join(name)
            .join(version.to_string());
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Store package in cache
    pub async fn store(
        &self,
        name: &str,
        version: &Version,
        data: &[u8],
    ) -> Result<PathBuf, PkgError> {
        let path = self
            .cache_dir
            .join("packages")
            .join(name)
            .join(version.to_string());

        tokio::fs::create_dir_all(&path).await?;

        // TODO: Extract tarball

        Ok(path)
    }
}

/// Load manifest from path
pub fn load_manifest(path: &Path) -> Result<Manifest, PkgError> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| PkgError::InvalidManifest(e.to_string()))
}

/// Save manifest to path
pub fn save_manifest(manifest: &Manifest, path: &Path) -> Result<(), PkgError> {
    let content = toml::to_string_pretty(manifest)
        .map_err(|e| PkgError::InvalidManifest(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
std = "0.1"
"#;
        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-app");
        assert!(manifest.dependencies.contains_key("std"));
    }
}
