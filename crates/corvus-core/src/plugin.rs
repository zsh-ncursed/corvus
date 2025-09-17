use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;
use log;

#[derive(Deserialize, Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
}

pub fn discover_plugins() -> Vec<Plugin> {
    let mut plugins = Vec::new();

    let local_plugins_dir = PathBuf::from("./test-plugins");

    let plugins_dir = if local_plugins_dir.exists() {
        local_plugins_dir
    } else {
        if let Some(proj_dirs) = ProjectDirs::from("com", "Corvus", "Corvus") {
            proj_dirs.config_dir().join("plugins")
        } else {
            return plugins;
        }
    };

    if !plugins_dir.exists() {
        if let Err(e) = fs::create_dir_all(&plugins_dir) {
            log::error!("Failed to create plugins directory: {}", e);
            return plugins;
        }
    }

    match fs::read_dir(plugins_dir) {
        Ok(entries) => {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.toml");
                    if manifest_path.exists() {
                        match fs::read_to_string(&manifest_path) {
                            Ok(content) => {
                                match toml::from_str::<PluginManifest>(&content) {
                                    Ok(manifest) => {
                                        plugins.push(Plugin {
                                            manifest,
                                            path: path.clone(),
                                            enabled: true, // Default to enabled
                                        });
                                    }
                                    Err(e) => log::error!("Failed to parse plugin manifest at {:?}: {}", manifest_path, e),
                                }
                            }
                            Err(e) => log::error!("Failed to read plugin manifest at {:?}: {}", manifest_path, e),
                        }
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to read plugins directory: {}", e),
    }


    plugins
}
