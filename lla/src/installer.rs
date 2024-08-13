use crate::error::{LlaError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct PluginInstaller {
    plugins_dir: PathBuf,
}

impl PluginInstaller {
    pub fn new(plugins_dir: &Path) -> Self {
        PluginInstaller {
            plugins_dir: plugins_dir.to_path_buf(),
        }
    }

    pub fn install_from_git(&self, url: &str) -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let repo_name = url
            .split('/')
            .last()
            .ok_or_else(|| LlaError::Plugin(format!("Invalid GitHub URL: {}", url)))?;

        let output = Command::new("git")
            .args(&["clone", url])
            .current_dir(&temp_dir)
            .output()?;

        if !output.status.success() {
            return Err(LlaError::Plugin(format!(
                "Failed to clone repository: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let repo_dir = temp_dir.path().join(repo_name);
        self.install_plugins(&repo_dir)
    }

    pub fn install_from_directory(&self, dir: &str) -> Result<()> {
        let source_dir = Path::new(dir);
        self.install_plugins(source_dir)
    }

    fn install_plugins(&self, root_dir: &Path) -> Result<()> {
        let plugin_dirs = self.find_plugin_directories(root_dir)?;

        if plugin_dirs.is_empty() {
            return Err(LlaError::Plugin("No plugins found".to_string()));
        }

        for plugin_dir in plugin_dirs {
            self.build_and_install_plugin(&plugin_dir)?;
        }

        Ok(())
    }

    fn find_plugin_directories(&self, root_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut plugin_dirs = Vec::new();

        for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && path.join("Cargo.toml").exists() {
                let cargo_toml = fs::read_to_string(path.join("Cargo.toml"))?;
                if cargo_toml.contains("lla_plugin_interface") {
                    plugin_dirs.push(path.to_path_buf());
                }
            }
        }

        Ok(plugin_dirs)
    }

    fn build_and_install_plugin(&self, source_dir: &Path) -> Result<()> {
        println!("Building plugin in {:?}", source_dir);

        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(source_dir)
            .output()?;

        if !output.status.success() {
            return Err(LlaError::Plugin(format!(
                "Failed to build plugin: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let target_dir = source_dir.join("target").join("release");
        let plugin_files: Vec<_> = target_dir
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib")
            })
            .collect();

        if plugin_files.is_empty() {
            return Err(LlaError::Plugin("No plugin files found".to_string()));
        }

        for plugin_file in plugin_files {
            let dest_path = self.plugins_dir.join(plugin_file.file_name());
            fs::copy(plugin_file.path(), &dest_path)?;
            println!("Plugin installed successfully: {:?}", dest_path);
        }

        Ok(())
    }
}
