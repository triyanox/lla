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
            .args(["clone", url])
            .current_dir(&temp_dir)
            .output()?;
        if !output.status.success() {
            return Err(LlaError::Plugin(format!(
                "Failed to clone repository: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        self.install_plugins(&temp_dir.path().join(repo_name))
    }

    pub fn install_from_directory(&self, dir: &str) -> Result<()> {
        let source_dir = PathBuf::from(dir.trim_end_matches('/'))
            .canonicalize()
            .map_err(|_| LlaError::Plugin(format!("Directory not found: {}", dir)))?;

        if !source_dir.exists() || !source_dir.is_dir() {
            return Err(LlaError::Plugin(format!("Not a valid directory: {}", dir)));
        }

        self.install_plugins(&source_dir)
    }

    fn install_plugins(&self, root_dir: &Path) -> Result<()> {
        let plugin_dirs = self.find_plugin_directories(root_dir)?;
        if plugin_dirs.is_empty() {
            return Err(LlaError::Plugin(format!(
                "No plugins found in {:?}",
                root_dir
            )));
        }
        let mut success = false;
        for plugin_dir in plugin_dirs {
            match self.build_and_install_plugin(&plugin_dir) {
                Ok(_) => {
                    println!(
                        "✅ Successfully built and installed plugin from {:?}",
                        plugin_dir
                    );
                    success = true;
                }
                Err(e) => {
                    eprintln!("❌ Failed to build plugin in {:?}", plugin_dir);
                    eprintln!("   Error: {}", e);
                }
            }
        }
        if success {
            Ok(())
        } else {
            Err(LlaError::Plugin(
                "No plugins were successfully installed".to_string(),
            ))
        }
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
                                            "🔍 Plugin is in workspace member directory at {:?}",
                                            parent
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

    fn find_plugin_directories(&self, root_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut plugin_dirs = Vec::new();
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
                    match fs::read_to_string(&cargo_toml) {
                        Ok(contents) if contents.contains("lla_plugin_interface") => {
                            println!("🔍 Found plugin project in {:?}", path);
                            plugin_dirs.push(path.to_path_buf());
                        }
                        Ok(_) => continue,
                        Err(e) => eprintln!("Failed to read {:?}: {}", cargo_toml, e),
                    }
                }
            }
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

    fn build_and_install_plugin(&self, plugin_dir: &Path) -> Result<()> {
        println!("\n🔨 Building plugin in {:?}", plugin_dir);
        let plugin_name = plugin_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| LlaError::Plugin("Invalid plugin directory name".to_string()))?;

        let (build_dir, build_args) =
            if let Some(workspace_root) = self.is_workspace_member(plugin_dir)? {
                println!("   📦 Building as workspace member");
                (
                    workspace_root,
                    vec!["build", "--release", "-p", plugin_name],
                )
            } else {
                println!("   📦 Building as standalone plugin");
                (plugin_dir.to_path_buf(), vec!["build", "--release"])
            };
        let _ = Command::new("cargo")
            .args(["clean"])
            .current_dir(&build_dir)
            .output();

        let output = Command::new("cargo")
            .args(&build_args)
            .current_dir(&build_dir)
            .output()?;

        if !output.status.success() {
            println!("   ❌ Build failed!");
            println!("   🔍 Build output:");
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            println!("   ⚠️ Build errors:");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(LlaError::Plugin("Build failed".to_string()));
        }

        let target_dir = build_dir.join("target").join("release");
        let plugin_files = self.find_plugin_files(&target_dir, plugin_name)?;

        if plugin_files.is_empty() {
            return Err(LlaError::Plugin(format!(
                "No plugin files found for '{}' in {:?}. Make sure the plugin is configured as a cdylib.",
                plugin_name, target_dir
            )));
        }

        fs::create_dir_all(&self.plugins_dir)?;

        for plugin_file in plugin_files {
            let dest_path = self.plugins_dir.join(plugin_file.file_name().unwrap());
            fs::copy(&plugin_file, &dest_path)?;
            println!("   ✨ Plugin installed: {:?}", dest_path);
        }

        Ok(())
    }
}
