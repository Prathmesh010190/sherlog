use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::parsers::ParsedLog;

/// Describes what a plugin can do.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    Parser,
    Formatter,
}

/// Metadata for an installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
}

/// Trait for WASM-based log parsers provided by plugins.
pub trait PluginParser {
    fn parser_name(&mut self) -> String;
    fn can_parse(&mut self, input: &str) -> bool;
    fn parse(&mut self, input: &str) -> Vec<ParsedLog>;
}

/// Trait for WASM-based output formatters provided by plugins.
pub trait PluginFormatter {
    fn formatter_name(&mut self) -> String;
    fn format_output(&mut self, json_input: &str) -> Result<String>;
}

/// A loaded WASM plugin using extism.
pub struct WasmPlugin {
    pub manifest: PluginManifest,
    plugin: extism::Plugin,
}

impl WasmPlugin {
    /// Load a WASM plugin from a file.
    /// Reads an optional `.json` manifest sidecar with the same stem.
    pub fn load(wasm_path: &Path) -> Result<Self> {
        let manifest_path = wasm_path.with_extension("json");
        let manifest: PluginManifest = if manifest_path.exists() {
            let data = fs::read_to_string(&manifest_path)
                .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
            serde_json::from_str(&data)
                .with_context(|| format!("Invalid manifest: {}", manifest_path.display()))?
        } else {
            let name = wasm_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            PluginManifest {
                name,
                version: "0.0.0".to_string(),
                description: String::new(),
                capabilities: vec![PluginCapability::Parser],
            }
        };

        let wasm = extism::Wasm::file(wasm_path);
        let ext_manifest = extism::Manifest::new([wasm]);
        let plugin = extism::Plugin::new(ext_manifest, [], true)
            .with_context(|| format!("Failed to load WASM plugin: {}", wasm_path.display()))?;

        Ok(Self { manifest, plugin })
    }

    /// Call a named export function with string I/O.
    pub fn call_func(&mut self, func: &str, input: &str) -> Result<String> {
        let output: String = self
            .plugin
            .call(func, input)
            .with_context(|| format!("Plugin function '{func}' failed"))?;
        Ok(output)
    }
}

impl PluginParser for WasmPlugin {
    fn parser_name(&mut self) -> String {
        self.call_func("name", "")
            .unwrap_or_else(|_| self.manifest.name.clone())
    }

    fn can_parse(&mut self, input: &str) -> bool {
        self.call_func("can_parse", input)
            .map(|r| r.trim() == "true")
            .unwrap_or(false)
    }

    fn parse(&mut self, input: &str) -> Vec<ParsedLog> {
        self.call_func("parse", input)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }
}

impl PluginFormatter for WasmPlugin {
    fn formatter_name(&mut self) -> String {
        self.call_func("formatter_name", "")
            .unwrap_or_else(|_| self.manifest.name.clone())
    }

    fn format_output(&mut self, json_input: &str) -> Result<String> {
        self.call_func("format", json_input)
    }
}

/// Manages plugin installation, listing, and removal.
pub struct PluginManager {
    plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let dir = Config::config_dir()?.join("plugins");
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create plugins dir: {}", dir.display()))?;
        Ok(Self { plugins_dir: dir })
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// List metadata for all installed plugins without fully loading WASM.
    pub fn list_manifests(&self) -> Result<Vec<PluginManifest>> {
        let mut manifests = Vec::new();
        if !self.plugins_dir.exists() {
            return Ok(manifests);
        }
        for entry in fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                let manifest_path = path.with_extension("json");
                let manifest = if manifest_path.exists() {
                    let data = fs::read_to_string(&manifest_path)?;
                    serde_json::from_str(&data)?
                } else {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    PluginManifest {
                        name,
                        version: "0.0.0".to_string(),
                        description: String::new(),
                        capabilities: vec![PluginCapability::Parser],
                    }
                };
                manifests.push(manifest);
            }
        }
        Ok(manifests)
    }

    /// Download and install a plugin from a URL.
    pub async fn install(&self, url: &str) -> Result<PluginManifest> {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to download plugin")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download plugin: HTTP {}", response.status());
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read plugin data")?;

        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("plugin.wasm")
            .trim_end_matches(".wasm")
            .to_string();

        let wasm_path = self.plugins_dir.join(format!("{name}.wasm"));
        fs::write(&wasm_path, &bytes)
            .with_context(|| format!("Failed to save plugin: {}", wasm_path.display()))?;

        match WasmPlugin::load(&wasm_path) {
            Ok(plugin) => Ok(plugin.manifest),
            Err(e) => {
                let _ = fs::remove_file(&wasm_path);
                Err(e).context("Downloaded file is not a valid WASM plugin")
            }
        }
    }

    /// Remove an installed plugin by name.
    pub fn remove(&self, name: &str) -> Result<()> {
        let wasm_path = self.plugins_dir.join(format!("{name}.wasm"));
        if !wasm_path.exists() {
            anyhow::bail!("Plugin not found: {name}");
        }
        fs::remove_file(&wasm_path)
            .with_context(|| format!("Failed to remove: {}", wasm_path.display()))?;
        let _ = fs::remove_file(self.plugins_dir.join(format!("{name}.json")));
        Ok(())
    }

    /// Load all installed plugins that have parser capability.
    pub fn load_parser_plugins(&self) -> Vec<WasmPlugin> {
        let mut parsers = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    if let Ok(plugin) = WasmPlugin::load(&path) {
                        if plugin
                            .manifest
                            .capabilities
                            .contains(&PluginCapability::Parser)
                        {
                            parsers.push(plugin);
                        }
                    }
                }
            }
        }
        parsers
    }
}
