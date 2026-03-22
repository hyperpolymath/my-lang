//! Configuration module for My SSG
//!
//! Handles loading and parsing site configuration.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    /// Site metadata
    pub site: SiteConfig,
    /// Build configuration
    pub build: BuildConfig,
    /// Feature flags
    pub features: FeatureConfig,
    /// Custom variables
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub content_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub static_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub syntax_highlighting: bool,
    pub ai_summaries: bool,
    pub minify_html: bool,
    pub minify_css: bool,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    ParseError(String),
}

impl Default for Config {
    fn default() -> Self {
        Config {
            site: SiteConfig {
                title: "My Site".to_string(),
                description: "A site built with My SSG".to_string(),
                base_url: "https://example.com".to_string(),
                language: "en".to_string(),
                author: None,
            },
            build: BuildConfig {
                content_dir: PathBuf::from("content"),
                templates_dir: PathBuf::from("templates"),
                static_dir: PathBuf::from("static"),
                output_dir: PathBuf::from("_site"),
            },
            features: FeatureConfig {
                syntax_highlighting: true,
                ai_summaries: false,
                minify_html: false,
                minify_css: false,
            },
            custom: HashMap::new(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML-like file
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse configuration from string content
    fn parse(content: &str) -> Result<Self, ConfigError> {
        let mut config = Config::default();
        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = &line[1..line.len() - 1];
                continue;
            }

            // Key-value pair
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match current_section {
                    "site" => match key {
                        "title" => config.site.title = value.to_string(),
                        "description" => config.site.description = value.to_string(),
                        "base_url" => config.site.base_url = value.to_string(),
                        "language" => config.site.language = value.to_string(),
                        "author" => config.site.author = Some(value.to_string()),
                        _ => {}
                    },
                    "build" => match key {
                        "content_dir" => config.build.content_dir = PathBuf::from(value),
                        "templates_dir" => config.build.templates_dir = PathBuf::from(value),
                        "static_dir" => config.build.static_dir = PathBuf::from(value),
                        "output_dir" => config.build.output_dir = PathBuf::from(value),
                        _ => {}
                    },
                    "features" => match key {
                        "syntax_highlighting" => {
                            config.features.syntax_highlighting = value == "true"
                        }
                        "ai_summaries" => config.features.ai_summaries = value == "true",
                        "minify_html" => config.features.minify_html = value == "true",
                        "minify_css" => config.features.minify_css = value == "true",
                        _ => {}
                    },
                    "custom" => {
                        config.custom.insert(key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.site.title, "My Site");
        assert_eq!(config.build.output_dir, PathBuf::from("_site"));
    }

    #[test]
    fn test_parse_config() {
        let content = r#"
[site]
title = "Test Site"
description = "A test"

[build]
output_dir = "dist"

[features]
syntax_highlighting = true
"#;
        let config = Config::parse(content).unwrap();
        assert_eq!(config.site.title, "Test Site");
        assert_eq!(config.build.output_dir, PathBuf::from("dist"));
        assert!(config.features.syntax_highlighting);
    }
}
