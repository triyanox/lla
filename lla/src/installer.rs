use crate::error::{LlaError, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use toml::{self, Value};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone)]
pub enum PluginSource {
    Git { url: String },
    Local { directory: String },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    name: String,
    version: String,
    source: PluginSource,
    installed_at: String,
    last_updated: String,
    repository_name: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct MetadataStore {
    plugins: HashMap<String, PluginMetadata>,
}

impl PluginMetadata {
    fn new(
        name: String,
        version: String,
        source: PluginSource,
        repository_name: Option<String>,
    ) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            name,
            version,
            source,
            installed_at: now.clone(),
            last_updated: now,
            repository_name,
        }
    }

    fn update_timestamp(&mut self) {
        self.last_updated = chrono::Local::now().to_rfc3339();
    }
}

#[derive(Default)]
struct InstallSummary {
    successful: Vec<(String, String)>,
    failed: Vec<(String, String)>,
}

impl InstallSummary {
    fn add_success(&mut self, name: String, version: String) {
        self.successful.push((name, version));
    }

    fn add_failure(&mut self, name: String, error: String) {
        self.failed.push((name, error));
    }

    fn display(&self) {
        if !self.successful.is_empty() {
            println!("\n{} Successfully installed:", style("✨").green());
            for (name, version) in &self.successful {
                println!("   {} {} ({})", style("✓").green(), name, version);
            }
        }

        if !self.failed.is_empty() {
            println!("\n{} Failed to install:", style("❌").red());
            for (name, error) in &self.failed {
                println!("   {} {} ({})", style("✗").red(), name, error);
            }
        }
    }
}

pub struct PluginInstaller {
    plugins_dir: PathBuf,
}

impl PluginInstaller {
    pub fn new(plugins_dir: &Path) -> Self {
        PluginInstaller {
            plugins_dir: plugins_dir.to_path_buf(),
        }
    }

    fn get_plugin_version(&self, plugin_dir: &Path) -> Result<String> {
        let cargo_toml_path = plugin_dir.join("Cargo.toml");
        let contents = fs::read_to_string(&cargo_toml_path)
            .map_err(|e| LlaError::Plugin(format!("Failed to read Cargo.toml: {}", e)))?;

        let cargo_toml: Value = toml::from_str(&contents)
            .map_err(|e| LlaError::Plugin(format!("Failed to parse Cargo.toml: {}", e)))?;

        let version = cargo_toml
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlaError::Plugin("No version found in Cargo.toml".to_string()))?;

        Ok(version.to_string())
    }

    fn load_metadata_store(&self) -> Result<MetadataStore> {
        let metadata_path = self.plugins_dir.join("metadata.toml");
        if !metadata_path.exists() {
            return Ok(MetadataStore::default());
        }

        let contents = fs::read_to_string(&metadata_path)
            .map_err(|e| LlaError::Plugin(format!("Failed to read metadata.toml: {}", e)))?;

        toml::from_str(&contents)
            .map_err(|e| LlaError::Plugin(format!("Failed to parse metadata.toml: {}", e)))
    }

    fn save_metadata_store(&self, store: &MetadataStore) -> Result<()> {
        let metadata_path = self.plugins_dir.join("metadata.toml");
        fs::create_dir_all(&self.plugins_dir)?;

        let toml_string = toml::to_string_pretty(store)
            .map_err(|e| LlaError::Plugin(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_path, toml_string)
            .map_err(|e| LlaError::Plugin(format!("Failed to write metadata.toml: {}", e)))
    }

    fn update_plugin_metadata(&self, plugin_name: &str, metadata: PluginMetadata) -> Result<()> {
        let mut store = self.load_metadata_store()?;
        store.plugins.insert(plugin_name.to_string(), metadata);
        self.save_metadata_store(&store)
    }

    fn create_progress_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{prefix:.green} {spinner:.green} [{bar:30.cyan/blue}] {msg} {bytes:>8} ({elapsed})")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .progress_chars("━━╾─")
    }

    fn update_progress(pb: &ProgressBar, percent: u64, msg: impl Into<String>, bytes: Option<u64>) {
        pb.set_position(percent);
        pb.set_message(msg.into());
        if let Some(bytes) = bytes {
            pb.set_length(bytes);
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    fn install_plugins(
        &self,
        root_dir: &Path,
        repo_info: Option<(&str, &str)>,
        pb: Option<&ProgressBar>,
    ) -> Result<()> {
        let plugin_dirs = self.find_plugin_directories(root_dir)?;
        if plugin_dirs.is_empty() {
            return Err(LlaError::Plugin(format!(
                "No plugins found in {:?}",
                root_dir
            )));
        }

        let mut summary = InstallSummary::default();
        let total_plugins = plugin_dirs.len();

        if let Some(pb) = pb {
            Self::update_progress(pb, 5, format!("found {} plugin(s)", total_plugins), None);
        }

        let progress_per_plugin = 95.0 / total_plugins as f64;

        for (idx, plugin_dir) in plugin_dirs.iter().enumerate() {
            let plugin_name = plugin_dir
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| LlaError::Plugin("Invalid plugin directory name".to_string()))?
                .to_string();

            let start_progress = 5 + (progress_per_plugin * idx as f64) as u64;
            let end_progress = 5 + (progress_per_plugin * (idx + 1) as f64) as u64;

            if let Some(pb) = pb {
                Self::update_progress(
                    pb,
                    start_progress,
                    format!("installing {}", plugin_name),
                    None,
                );
            }

            match self.build_and_install_plugin(plugin_dir, pb, Some(start_progress)) {
                Ok(_) => {
                    let version = self.get_plugin_version(plugin_dir)?;
                    let metadata = if let Some((repo_name, url)) = repo_info {
                        PluginMetadata::new(
                            plugin_name.clone(),
                            version.clone(),
                            PluginSource::Git {
                                url: url.to_string(),
                            },
                            Some(repo_name.to_string()),
                        )
                    } else {
                        PluginMetadata::new(
                            plugin_name.clone(),
                            version.clone(),
                            PluginSource::Local {
                                directory: root_dir.to_string_lossy().into_owned(),
                            },
                            None,
                        )
                    };

                    if let Err(e) = self.update_plugin_metadata(&plugin_name, metadata) {
                        summary.add_failure(plugin_name, format!("metadata error: {}", e));
                    } else {
                        summary.add_success(plugin_name, version);
                    }
                }
                Err(e) => {
                    summary.add_failure(plugin_name.clone(), e.to_string());
                    if let Some(pb) = pb {
                        Self::update_progress(
                            pb,
                            end_progress,
                            format!("❌ {} failed", plugin_name),
                            None,
                        );
                    }
                }
            }

            if let Some(pb) = pb {
                Self::update_progress(
                    pb,
                    end_progress,
                    format!("processed {}/{}", idx + 1, total_plugins),
                    None,
                );
            }
        }

        if let Some(pb) = pb {
            if summary.failed.is_empty() {
                pb.finish_with_message(format!(
                    "✨ installed {} plugin(s)",
                    summary.successful.len()
                ));
            } else {
                pb.finish_with_message(format!(
                    "⚠️ installed {}/{} plugin(s)",
                    summary.successful.len(),
                    total_plugins
                ));
            }
        }

        summary.display();

        if !summary.failed.is_empty() {
            Err(LlaError::Plugin(format!(
                "{}/{} plugins failed to install",
                summary.failed.len(),
                total_plugins
            )))
        } else {
            Ok(())
        }
    }

    pub fn install_from_git(&self, url: &str) -> Result<()> {
        println!("{} Installing from Git", style("📦").green());
        let m = MultiProgress::new();
        let pb = m.add(ProgressBar::new(100));
        pb.set_style(Self::create_progress_style());
        pb.set_prefix("🔄");
        pb.enable_steady_tick(Duration::from_millis(80));

        Self::update_progress(&pb, 0, "cloning repository", None);
        let temp_dir = tempfile::tempdir()?;
        let repo_name = url
            .split('/')
            .last()
            .ok_or_else(|| LlaError::Plugin(format!("Invalid GitHub URL: {}", url)))?
            .trim_end_matches(".git");

        let size_output = Command::new("git").args(["ls-remote", url]).output().ok();

        let repo_size = size_output
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.len() as u64 * 100)
            .unwrap_or(0);

        Self::update_progress(&pb, 10, "downloading", Some(repo_size));

        let output = Command::new("git")
            .args(["clone", "--quiet", "--progress", url])
            .current_dir(&temp_dir)
            .output()?;

        if !output.status.success() {
            pb.finish_with_message("❌ clone failed");
            return Err(LlaError::Plugin(format!(
                "Failed to clone repository: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Self::update_progress(&pb, 30, "finding plugins", None);
        self.install_plugins(
            &temp_dir.path().join(repo_name),
            Some((repo_name, url)),
            Some(&pb),
        )
    }

    pub fn install_from_directory(&self, dir: &str) -> Result<()> {
        println!("{} Installing from directory", style("📦").green());
        let m = MultiProgress::new();
        let pb = m.add(ProgressBar::new(100));
        pb.set_style(Self::create_progress_style());
        pb.set_prefix("🔄");
        pb.enable_steady_tick(Duration::from_millis(80));

        pb.set_message("checking source");
        let source_dir = PathBuf::from(dir.trim_end_matches('/'))
            .canonicalize()
            .map_err(|_| LlaError::Plugin(format!("Directory not found: {}", dir)))?;

        if !source_dir.exists() || !source_dir.is_dir() {
            pb.finish_with_message("❌ invalid directory");
            return Err(LlaError::Plugin(format!("Not a valid directory: {}", dir)));
        }

        pb.inc(30);
        pb.set_message("finding plugins");
        self.install_plugins(&source_dir, None, Some(&pb))
    }

    fn is_workspace_member(&self, plugin_dir: &Path) -> Result<Option<PathBuf>> {
        let mut current_dir = plugin_dir.to_path_buf();
        while let Some(parent) = current_dir.parent() {
            let workspace_cargo = parent.join("Cargo.toml");
            if workspace_cargo.exists() {
                if let Ok(contents) = fs::read_to_string(&workspace_cargo) {
                    if contents.contains("[workspace]") {
                        if let Ok(rel_path) = plugin_dir.strip_prefix(parent) {
                            let rel_path_str = rel_path.to_string_lossy();

                            if contents.contains(&format!("\"{}\"", rel_path_str))
                                || contents.contains(&format!("'{}'", rel_path_str))
                            {
                                println!("🔍 Plugin is direct workspace member at {:?}", parent);
                                return Ok(Some(parent.to_path_buf()));
                            }
                            if contents.contains("members = [") {
                                let patterns = [
                                    format!(
                                        "\"{}/*\"",
                                        rel_path_str.split('/').next().unwrap_or("")
                                    ),
                                    format!("'{}/*'", rel_path_str.split('/').next().unwrap_or("")),
                                    format!(
                                        "\"{}/\"",
                                        rel_path_str.split('/').next().unwrap_or("")
                                    ),
                                    format!("'{}/", rel_path_str.split('/').next().unwrap_or("")),
                                ];

                                for pattern in patterns {
                                    if contents.contains(&pattern) {
                                        println!(
                                            "🔍 Plugin is in workspace member directory at {}",
                                            parent.display()
                                        );
                                        return Ok(Some(parent.to_path_buf()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            current_dir = parent.to_path_buf();
        }
        println!("🔍 Plugin is standalone");
        Ok(None)
    }

    fn get_display_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn find_plugin_directories(&self, root_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut plugin_dirs = Vec::new();
        let mut found_plugins = Vec::new();

        let workspace_cargo = root_dir.join("Cargo.toml");
        if workspace_cargo.exists() {
            if let Ok(contents) = fs::read_to_string(&workspace_cargo) {
                if contents.contains("[workspace]") {
                    for entry in WalkDir::new(root_dir)
                        .follow_links(true)
                        .min_depth(1)
                        .max_depth(3)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        let path = entry.path();
                        if path.is_dir() {
                            let cargo_toml = path.join("Cargo.toml");
                            if cargo_toml.exists() {
                                if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                                    if contents.contains("lla_plugin_interface") {
                                        found_plugins.push(Self::get_display_name(path));
                                        plugin_dirs.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                    if !found_plugins.is_empty() {
                        println!(
                            "🔍 Found plugins: {}",
                            style(found_plugins.join(", ")).cyan()
                        );
                        return Ok(plugin_dirs);
                    }
                }
            }
        }

        for entry in WalkDir::new(root_dir)
            .follow_links(true)
            .min_depth(0)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.exists() {
                    if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                        if contents.contains("lla_plugin_interface") {
                            found_plugins.push(Self::get_display_name(path));
                            plugin_dirs.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        if !found_plugins.is_empty() {
            println!(
                "🔍 Found plugins: {}",
                style(found_plugins.join(", ")).cyan()
            );
        }

        Ok(plugin_dirs)
    }

    fn find_plugin_files(&self, target_dir: &Path, plugin_name: &str) -> Result<Vec<PathBuf>> {
        let mut plugin_files = Vec::new();
        if let Ok(entries) = target_dir.read_dir() {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let is_plugin = match std::env::consts::OS {
                    "macos" => file_name.contains(plugin_name) && file_name.ends_with(".dylib"),
                    "linux" => file_name.contains(plugin_name) && file_name.ends_with(".so"),
                    "windows" => file_name.contains(plugin_name) && file_name.ends_with(".dll"),
                    _ => false,
                };

                if is_plugin {
                    plugin_files.push(path);
                }
            }
        }
        Ok(plugin_files)
    }

    fn build_and_install_plugin(
        &self,
        plugin_dir: &Path,
        pb: Option<&ProgressBar>,
        base_progress: Option<u64>,
    ) -> Result<()> {
        let plugin_name = Self::get_display_name(plugin_dir);
        let base = base_progress.unwrap_or(0);
        let step = 15;

        if let Some(pb) = pb {
            Self::update_progress(pb, base + 1, format!("building {}", plugin_name), None);
        }

        let (build_dir, build_args) =
            if let Some(workspace_root) = self.is_workspace_member(plugin_dir)? {
                (
                    workspace_root,
                    vec!["build", "--release", "-p", &plugin_name],
                )
            } else {
                (plugin_dir.to_path_buf(), vec!["build", "--release"])
            };

        let output = Command::new("cargo")
            .args(&build_args)
            .current_dir(&build_dir)
            .output()?;

        if !output.status.success() {
            if let Some(pb) = pb {
                pb.finish_with_message(format!("❌ {} build failed", plugin_name));
            }
            return Err(LlaError::Plugin(format!(
                "Build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        if let Some(pb) = pb {
            Self::update_progress(
                pb,
                base + step * 2,
                format!("installing {}", plugin_name),
                None,
            );
        }

        let target_dir = build_dir.join("target").join("release");
        let plugin_files = self.find_plugin_files(&target_dir, &plugin_name)?;

        if plugin_files.is_empty() {
            if let Some(pb) = pb {
                pb.finish_with_message(format!("❌ {} no files found", plugin_name));
            }
            return Err(LlaError::Plugin(format!(
                "No plugin files found for '{}'",
                plugin_name
            )));
        }

        fs::create_dir_all(&self.plugins_dir)?;

        for plugin_file in plugin_files {
            let dest_path = self.plugins_dir.join(plugin_file.file_name().unwrap());
            fs::copy(&plugin_file, &dest_path)?;
        }

        if let Some(pb) = pb {
            Self::update_progress(
                pb,
                base + step * 4,
                format!("✓ {} installed", plugin_name),
                None,
            );
        }

        Ok(())
    }

    pub fn update_plugins(&self, plugin_name: Option<&str>) -> Result<()> {
        let store = self.load_metadata_store()?;
        if store.plugins.is_empty() {
            return Err(LlaError::Plugin(
                "No plugins are currently installed".to_string(),
            ));
        }

        let plugins: Vec<_> = if let Some(name) = plugin_name {
            store.plugins.iter().filter(|(n, _)| *n == name).collect()
        } else {
            store.plugins.iter().collect()
        };

        if plugins.is_empty() {
            return Err(LlaError::Plugin(format!(
                "Plugin '{}' not found",
                plugin_name.unwrap_or_default()
            )));
        }

        println!("{} {} plugin(s)", style("📦").green(), plugins.len());

        let m = MultiProgress::new();
        let sty = Self::create_progress_style();

        let mut success = false;
        let handles: Vec<std::thread::JoinHandle<()>> = vec![];

        for (name, metadata) in plugins {
            let pb = m.add(ProgressBar::new(100));
            pb.set_style(sty.clone());
            pb.set_prefix(format!("🔄 {}", name));
            pb.enable_steady_tick(Duration::from_millis(80));
            Self::update_progress(&pb, 0, "preparing", None);

            match &metadata.source {
                PluginSource::Git { url } => {
                    Self::update_progress(&pb, 10, "cloning repository", None);
                    let temp_dir = match tempfile::tempdir() {
                        Ok(dir) => dir,
                        Err(e) => {
                            pb.finish_with_message(format!("❌ failed: {}", e));
                            continue;
                        }
                    };

                    let output = Command::new("git")
                        .args(["clone", "--quiet", url])
                        .current_dir(&temp_dir)
                        .output()?;

                    if !output.status.success() {
                        pb.finish_with_message("❌ clone failed");
                        continue;
                    }

                    Self::update_progress(&pb, 30, "locating plugin", None);
                    let repo_name = url
                        .split('/')
                        .last()
                        .map(|n| n.trim_end_matches(".git"))
                        .unwrap_or(name);

                    let repo_dir = temp_dir.path().join(repo_name);
                    let plugin_dirs = self.find_plugin_directories(&repo_dir)?;

                    if let Some(plugin_dir) = plugin_dirs.iter().find(|dir| {
                        dir.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == name)
                            .unwrap_or(false)
                    }) {
                        match self.build_and_install_plugin(plugin_dir, Some(&pb), None) {
                            Ok(_) => {
                                let new_version = self.get_plugin_version(plugin_dir)?;
                                let mut updated_metadata = metadata.clone();

                                if new_version != metadata.version {
                                    pb.finish_with_message(format!(
                                        "✨ {} → {}",
                                        metadata.version, new_version
                                    ));
                                } else {
                                    pb.finish_with_message(format!("✨ {}", new_version));
                                }

                                updated_metadata.version = new_version;
                                updated_metadata.update_timestamp();
                                self.update_plugin_metadata(name, updated_metadata)?;
                                success = true;
                            }
                            Err(e) => {
                                pb.finish_with_message(format!("❌ build failed: {}", e));
                            }
                        }
                    } else {
                        pb.finish_with_message("❌ plugin not found in repo");
                    }
                }
                PluginSource::Local { directory } => {
                    Self::update_progress(&pb, 10, "checking source", None);
                    let source_dir = PathBuf::from(directory);

                    if !source_dir.exists() {
                        pb.finish_with_message("❌ source not found");
                        continue;
                    }

                    pb.inc(40);
                    match self.build_and_install_plugin(&source_dir, Some(&pb), None) {
                        Ok(_) => {
                            let new_version = self.get_plugin_version(&source_dir)?;
                            let mut updated_metadata = metadata.clone();

                            if new_version != metadata.version {
                                pb.finish_with_message(format!(
                                    "✨ {} → {}",
                                    metadata.version, new_version
                                ));
                            } else {
                                pb.finish_with_message(format!("✨ {}", new_version));
                            }

                            updated_metadata.version = new_version;
                            updated_metadata.update_timestamp();
                            self.update_plugin_metadata(name, updated_metadata)?;
                            success = true;
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("❌ build failed: {}", e));
                        }
                    }
                }
            }
        }

        for handle in handles {
            let _ = handle.join();
        }

        if success {
            println!("\n{} Update complete", style("✅").green());
            Ok(())
        } else if let Some(name) = plugin_name {
            Err(LlaError::Plugin(format!(
                "Failed to update plugin: {}",
                name
            )))
        } else {
            Err(LlaError::Plugin(
                "No plugins were successfully updated".to_string(),
            ))
        }
    }
}
