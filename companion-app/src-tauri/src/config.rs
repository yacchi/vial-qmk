use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    /// Application name pattern (substring match)
    pub app_pattern: String,
    /// Target default layer
    pub layer: u8,
    /// OS key override setting (optional)
    pub os_override: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub default_layer: u8,
    pub os_override: u8,
    pub app_rules: Vec<AppRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Active profile name
    pub active_profile: String,
    /// Auto-switch enabled
    pub auto_switch_enabled: bool,
    /// Profiles
    pub profiles: HashMap<String, Profile>,
    /// Additional config files to merge (for sync tools)
    #[serde(default)]
    pub include_configs: Vec<String>,
    /// Path to Vial application (optional)
    pub vial_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            "default".to_string(),
            Profile {
                name: "default".to_string(),
                default_layer: 0,
                os_override: 0,
                app_rules: vec![],
            },
        );

        Config {
            active_profile: "default".to_string(),
            auto_switch_enabled: false,
            profiles,
            include_configs: vec![],
            vial_path: None,
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kqb-companion")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config: {}", e),
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(Self::config_path(), content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    /// Load and merge additional config files specified in include_configs
    pub fn load_with_includes() -> Self {
        let mut config = Self::load();

        for include_path in config.include_configs.clone() {
            let path = PathBuf::from(&include_path);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(extra_config) = serde_json::from_str::<Config>(&content) {
                        // Merge profiles from included configs
                        for (name, profile) in extra_config.profiles {
                            config.profiles.entry(name).or_insert(profile);
                        }
                    }
                }
            }
        }

        config
    }

    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles.get(&self.active_profile)
    }
}
