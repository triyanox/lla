use crate::commands::args::Args;
use crate::error::{LlaError, Result};
use crate::utils::color::ColorState;
use colored::{ColoredString, Colorize};
use console::{style, Term};
use dialoguer::MultiSelect;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lla_plugin_utils::ui::components::LlaDialoguerTheme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
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
            println!("Successfully installed:");
            for (name, version) in &self.successful {
                println!(
                    "  {} {} v{}",
                    "✓".green(),
                    name.bright_blue(),
                    version.bright_black()
                );
            }
        }

        if !self.failed.is_empty() {
            if !self.successful.is_empty() {
                println!();
            }
            println!("Failed to install:");
            for (name, error) in &self.failed {
                println!(
                    "  {} {} - {}",
                    "✗".red(),
                    name.bright_blue(),
                    error.bright_black()
                );
            }
        }
    }
}

pub struct PluginInstaller {
    plugins_dir: PathBuf,
    color_state: ColorState,
}

impl PluginInstaller {
    pub fn new(plugins_dir: &Path, args: &Args) -> Self {
        PluginInstaller {
            plugins_dir: plugins_dir.to_path_buf(),
            color_state: ColorState::new(args),
        }
    }

    fn display_colored(&self, text: &str, color_fn: fn(&str) -> ColoredString) -> String {
        if self.color_state.is_enabled() {
            color_fn(text).to_string()
        } else {
            text.to_string()
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
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
    }

    fn select_plugins(&self, plugin_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
        if !atty::is(atty::Stream::Stdout) {
            return Ok(plugin_dirs.to_vec());
        }

        let plugin_names: Vec<String> = plugin_dirs
            .iter()
            .map(|p| {
                let name = Self::get_display_name(p);
                let version = self
                    .get_plugin_version(p)
                    .unwrap_or_else(|_| "unknown".to_string());
                format!("{} v{}", name, version)
            })
            .collect();

        if plugin_names.is_empty() {
            return Err(LlaError::Plugin("No plugins found".to_string()));
        }

        println!("\n{}", "Plugin Installation".cyan().bold());
        println!("{}\n", "Space to toggle, Enter to confirm".bright_black());

        let theme = LlaDialoguerTheme::default();

        let selections = MultiSelect::with_theme(&theme)
            .with_prompt("Select plugins to install")
            .items(&plugin_names)
            .defaults(&vec![false; plugin_names.len()])
            .interact_on(&Term::stderr())?;

        if selections.is_empty() {
            return Err(LlaError::Plugin("No plugins selected".to_string()));
        }

        Ok(selections
            .into_iter()
            .map(|i| plugin_dirs[i].clone())
            .collect())
    }

    pub fn install_from_git(&self, url: &str) -> Result<()> {
        println!("\n{}\n", "Installing from Git Repository".cyan().bold());
        let m = MultiProgress::new();

        let pb = m.add(ProgressBar::new(1));
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message("Cloning repository...");
        pb.enable_steady_tick(Duration::from_millis(80));

        let temp_dir = tempfile::tempdir()?;
        let repo_name = url
            .split('/')
            .last()
            .ok_or_else(|| LlaError::Plugin(format!("Invalid GitHub URL: {}", url)))?
            .trim_end_matches(".git");

        let mut child = Command::new("git")
            .args(["clone", "--quiet", url])
            .current_dir(&temp_dir)
            .spawn()?;

        let status = child.wait()?;
        if !status.success() {
            pb.finish_with_message("Clone failed");
            return Err(LlaError::Plugin("Failed to clone repository".to_string()));
        }

        pb.finish_and_clear();
        drop(pb);

        let result = self.install_plugins(
            &temp_dir.path().join(repo_name),
            Some((repo_name, url)),
            Some(&m),
        );

        m.clear()?;
        println!();

        result
    }

    pub fn install_from_directory(&self, dir: &str) -> Result<()> {
        println!("\n{}\n", "Installing from Directory".cyan().bold());
        let m = MultiProgress::new();

        let source_dir = PathBuf::from(dir.trim_end_matches('/'))
            .canonicalize()
            .map_err(|_| LlaError::Plugin(format!("Directory not found: {}", dir)))?;

        if !source_dir.exists() || !source_dir.is_dir() {
            return Err(LlaError::Plugin(format!("Not a valid directory: {}", dir)));
        }

        let result = self.install_plugins(&source_dir, None, Some(&m));

        m.clear()?;

        result
    }

    fn is_workspace_member(&self, plugin_dir: &Path) -> Result<Option<PathBuf>> {
        let mut current_dir = plugin_dir.to_path_buf();
        let plugin_name = Self::get_display_name(plugin_dir);

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
                                println!("🔍 Plugin is in a workspace member");
                                println!(
                                    "  ✓ Found {} in workspace at {:?}",
                                    plugin_name.bright_blue(),
                                    parent
                                );
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
                                        println!("🔍 Plugin is in a workspace member");
                                        println!(
                                            "  ✓ Found {} in workspace pattern {}",
                                            plugin_name.bright_blue(),
                                            pattern.bright_black()
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
        println!(" Plugin is standalone");
        println!(
            "  ℹ {} will be built independently",
            plugin_name.bright_blue()
        );
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
                                        let name = Self::get_display_name(path);
                                        if name != "lla_plugin_interface" {
                                            if let Ok(version) = self.get_plugin_version(path) {
                                                found_plugins
                                                    .push(format!("{} v{}", name, version));
                                                plugin_dirs.push(path.to_path_buf());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !found_plugins.is_empty() {
                        println!(
                            "🔍 Found plugins: {}",
                            if self.color_state.is_enabled() {
                                style(found_plugins.join(", ")).cyan().to_string()
                            } else {
                                found_plugins.join(", ")
                            }
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
                            let name = Self::get_display_name(path);
                            if name != "lla_plugin_interface" {
                                if let Ok(version) = self.get_plugin_version(path) {
                                    found_plugins.push(format!("{} v{}", name, version));
                                    plugin_dirs.push(path.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found_plugins.is_empty() {
            println!(
                "🔍 Found plugins: {}",
                if self.color_state.is_enabled() {
                    style(found_plugins.join(", ")).cyan().to_string()
                } else {
                    found_plugins.join(", ")
                }
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
        _base_progress: Option<u64>,
    ) -> Result<()> {
        let plugin_name = Self::get_display_name(plugin_dir);

        let (build_dir, build_args) = match self.is_workspace_member(plugin_dir)? {
            Some(workspace_root) => {
                if let Some(pb) = pb {
                    pb.set_message(format!("Building {} in workspace", plugin_name));
                }
                (
                    workspace_root,
                    vec!["build", "--release", "-p", &plugin_name],
                )
            }
            None => {
                if let Some(pb) = pb {
                    pb.set_message(format!("Building {}", plugin_name));
                }
                (plugin_dir.to_path_buf(), vec!["build", "--release"])
            }
        };

        let mut child = Command::new("cargo")
            .args(&build_args)
            .current_dir(&build_dir)
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(pb) = pb {
            if let Some(stderr) = child.stderr.take() {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.contains("Compiling") {
                            pb.set_message(format!("Building {}", plugin_name));
                        }
                    }
                }
            }
        }

        let status = child.wait()?;
        if !status.success() {
            return Err(LlaError::Plugin("Build failed".to_string()));
        }

        let target_dir = build_dir.join("target").join("release");
        let plugin_files = self.find_plugin_files(&target_dir, &plugin_name)?;

        if plugin_files.is_empty() {
            return Err(LlaError::Plugin(format!(
                "No plugin files found for '{}'",
                plugin_name
            )));
        }

        if let Some(pb) = pb {
            pb.set_message(format!("Installing {}", plugin_name));
        }

        fs::create_dir_all(&self.plugins_dir)?;

        for plugin_file in plugin_files.iter() {
            let dest_path = self.plugins_dir.join(plugin_file.file_name().unwrap());
            fs::copy(plugin_file, &dest_path)?;
        }

        println!(
            "  ✓ Successfully installed {}",
            self.display_colored(&plugin_name, |s| s.bright_blue())
        );
        Ok(())
    }

    fn install_plugins(
        &self,
        root_dir: &Path,
        repo_info: Option<(&str, &str)>,
        multi_progress: Option<&MultiProgress>,
    ) -> Result<()> {
        let plugin_dirs = self.find_plugin_directories(root_dir)?;
        if plugin_dirs.is_empty() {
            return Err(LlaError::Plugin(format!(
                "No plugins found in {:?}",
                root_dir
            )));
        }

        let selected_plugins = self.select_plugins(&plugin_dirs)?;
        let mut summary = InstallSummary::default();
        let total_plugins = selected_plugins.len();

        for plugin_dir in selected_plugins.iter() {
            let plugin_name = Self::get_display_name(plugin_dir);

            let progress_bar = if let Some(m) = multi_progress {
                let pb = m.add(ProgressBar::new(1));
                pb.set_style(Self::create_progress_style());
                pb.enable_steady_tick(Duration::from_millis(80));
                pb.set_message(format!("Setting up {}", plugin_name));
                Some(pb)
            } else {
                None
            };

            match self.build_and_install_plugin(plugin_dir, progress_bar.as_ref(), None) {
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
                        let canonical_path = plugin_dir.canonicalize().map_err(|e| {
                            LlaError::Plugin(format!("Failed to resolve plugin path: {}", e))
                        })?;
                        PluginMetadata::new(
                            plugin_name.clone(),
                            version.clone(),
                            PluginSource::Local {
                                directory: canonical_path.to_string_lossy().into_owned(),
                            },
                            None,
                        )
                    };

                    if let Err(e) = self.update_plugin_metadata(&plugin_name, metadata) {
                        summary.add_failure(plugin_name.clone(), format!("metadata error: {}", e));
                        if let Some(ref pb) = progress_bar {
                            pb.finish_with_message(format!("Failed to install {}", plugin_name));
                        }
                    } else {
                        summary.add_success(plugin_name.clone(), version.clone());
                        if let Some(ref pb) = progress_bar {
                            pb.finish_with_message(format!(
                                "✓ Installed {} v{}",
                                plugin_name, version
                            ));
                        }
                    }
                }
                Err(e) => {
                    summary.add_failure(plugin_name.clone(), e.to_string());
                    if let Some(ref pb) = progress_bar {
                        pb.finish_with_message(format!("✗ Failed to install {}", plugin_name));
                    }
                }
            }

            if let Some(ref pb) = progress_bar {
                pb.finish_and_clear();
            }
        }

        if let Some(m) = multi_progress {
            m.clear()?;
        }

        println!("\nInstallation Summary");
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
        let sty = ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

        let mut success = false;
        for (name, metadata) in plugins {
            let pb = m.add(ProgressBar::new(1));
            pb.set_style(sty.clone());
            pb.enable_steady_tick(Duration::from_millis(80));
            pb.set_message(format!("Updating {}", name));

            match &metadata.source {
                PluginSource::Git { url } => {
                    let temp_dir = match tempfile::tempdir() {
                        Ok(dir) => dir,
                        Err(e) => {
                            pb.finish_with_message(format!("✗ Failed: {}", e));
                            continue;
                        }
                    };

                    let output = Command::new("git")
                        .args(["clone", "--quiet", url])
                        .current_dir(&temp_dir)
                        .output()?;

                    if !output.status.success() {
                        pb.finish_with_message(format!("✗ Failed to clone {}", name));
                        continue;
                    }

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
                                        "✓ Updated {} {} → {}",
                                        name, metadata.version, new_version
                                    ));
                                } else {
                                    pb.finish_with_message(format!(
                                        "✓ {} is up to date ({})",
                                        name, new_version
                                    ));
                                }

                                updated_metadata.version = new_version;
                                updated_metadata.update_timestamp();
                                self.update_plugin_metadata(name, updated_metadata)?;
                                success = true;
                            }
                            Err(e) => {
                                pb.finish_with_message(format!(
                                    "✗ Failed to build {}: {}",
                                    name, e
                                ));
                            }
                        }
                    } else {
                        pb.finish_with_message(format!("✗ {} not found in repository", name));
                    }
                }
                PluginSource::Local { directory } => {
                    let source_dir = PathBuf::from(directory);

                    if !source_dir.exists() {
                        pb.finish_with_message(format!("✗ Source not found for {}", name));
                        continue;
                    }

                    match self.build_and_install_plugin(&source_dir, Some(&pb), None) {
                        Ok(_) => {
                            let new_version = self.get_plugin_version(&source_dir)?;
                            let mut updated_metadata = metadata.clone();

                            if new_version != metadata.version {
                                pb.finish_with_message(format!(
                                    "✓ Updated {} {} → {}",
                                    name, metadata.version, new_version
                                ));
                            } else {
                                pb.finish_with_message(format!(
                                    "✓ {} is up to date ({})",
                                    name, new_version
                                ));
                            }

                            updated_metadata.version = new_version;
                            updated_metadata.update_timestamp();
                            self.update_plugin_metadata(name, updated_metadata)?;
                            success = true;
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("✗ Failed to build {}: {}", name, e));
                        }
                    }
                }
            }
        }

        if success {
            Ok(())
        } else if let Some(name) = plugin_name {
            Err(LlaError::Plugin(format!("Failed to update {}", name)))
        } else {
            Err(LlaError::Plugin("No plugins were updated".to_string()))
        }
    }
}
