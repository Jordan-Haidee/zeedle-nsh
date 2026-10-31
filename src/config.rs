use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const LEGACY_STORAGE_KEYS: [&str; 6] = [
    "zeedle.music-directory",
    "zeedle.volume",
    "zeedle.play-mode",
    "zeedle.show-spectrum",
    "zeedle.language",
    "zeedle.light-theme",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMode {
    #[default]
    InOrder,
    Recursive,
    Random,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    #[default]
    BySongName,
    BySinger,
    ByDuration,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub song_dir: PathBuf,
    pub current_song_path: Option<PathBuf>,
    pub progress: f32,
    pub play_mode: PlayMode,
    pub sort_key: SortKey,
    pub sort_ascending: bool,
    pub lang: String,
    pub light_ui: bool,
    #[serde(default)]
    pub follow_system_theme: bool,
    pub volume: f32,
    #[serde(default)]
    pub show_spectrum: bool,
    #[serde(default)]
    pub current_page: i32,
    #[serde(default)]
    pub sidebar_expanded: bool,
    #[serde(default)]
    pub gallery_viewport_y: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            song_dir: home::home_dir().unwrap_or_default().join("Music"),
            current_song_path: None,
            progress: 0.0,
            play_mode: PlayMode::InOrder,
            sort_key: SortKey::BySongName,
            sort_ascending: true,
            lang: String::new(),
            light_ui: false,
            follow_system_theme: true,
            volume: 1.0,
            show_spectrum: true,
            current_page: 0,
            sidebar_expanded: false,
            gallery_viewport_y: 0.0,
        }
    }
}

impl Config {
    pub fn path_for_home(home: &Path) -> PathBuf {
        home.join(".config").join("zeedle").join("config.toml")
    }

    pub fn path() -> PathBuf {
        std::env::var_os("ZEEDLE_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::path_for_home(&home::home_dir().unwrap_or_default()))
    }

    pub fn load() -> Self {
        let path = Self::path();
        match Self::load_at(&path) {
            Ok(config) => {
                log::info!("loaded config state from {}", path.display());
                config
            }
            Err(error) => {
                log::error!("failed to load config {}: {error}", path.display());
                Self::default()
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::path();
        self.save_at(&path)?;
        log::info!("saved config to {}", path.display());
        Ok(())
    }

    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn load_at(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        match Self::from_toml(&text) {
            Ok(config) => Ok(config),
            Err(error) => {
                log::warn!("invalid config TOML at {}: {error}", path.display());
                Ok(Self::default())
            }
        }
    }

    pub fn save_at(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = self
            .to_toml()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        std::fs::write(path, text)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{Config, PlayMode, SortKey};

    static NEXT_PATH: AtomicU64 = AtomicU64::new(0);

    fn temporary_config_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "zeedle-config-test-{}-{}.toml",
            std::process::id(),
            NEXT_PATH.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn defaults_match_slint() {
        let config = Config::default();
        assert!(config.song_dir.ends_with("Music"));
        assert_eq!(config.current_song_path, None);
        assert_eq!(config.progress, 0.0);
        assert_eq!(config.play_mode, PlayMode::InOrder);
        assert_eq!(config.sort_key, SortKey::BySongName);
        assert!(config.sort_ascending);
        assert_eq!(config.lang, "");
        assert!(!config.light_ui);
        assert!(config.follow_system_theme);
        assert_eq!(config.volume, 1.0);
        assert!(config.show_spectrum);
        assert_eq!(config.current_page, 0);
        assert!(!config.sidebar_expanded);
        assert_eq!(config.gallery_viewport_y, 0.0);
    }

    #[test]
    fn toml_round_trip_keeps_slint_field_and_enum_names() {
        let config = Config::default();
        let text = config.to_toml().unwrap();
        assert!(text.contains("play_mode = \"InOrder\""));
        assert!(text.contains("sort_key = \"BySongName\""));
        assert_eq!(Config::from_toml(&text).unwrap(), config);
    }

    #[test]
    fn path_matches_slint_config_location() {
        assert_eq!(
            Config::path_for_home(Path::new("/home/tester")),
            PathBuf::from("/home/tester/.config/zeedle/config.toml")
        );
    }

    #[test]
    fn older_slint_config_uses_defaults_for_later_fields() {
        let text = r#"
song_dir = "/music"
current_song_path = "/music/song.mp3"
progress = 12.5
play_mode = "InOrder"
sort_key = "BySongName"
sort_ascending = true
lang = ""
light_ui = false
volume = 1.0
"#;

        let config = Config::from_toml(text).unwrap();

        assert!(!config.follow_system_theme);
        assert!(!config.show_spectrum);
        assert_eq!(config.current_page, 0);
        assert!(!config.sidebar_expanded);
        assert_eq!(config.gallery_viewport_y, 0.0);
    }

    #[test]
    fn missing_and_invalid_files_use_slint_defaults() {
        let missing = temporary_config_path();
        assert_eq!(Config::load_at(&missing).unwrap(), Config::default());

        let invalid = temporary_config_path();
        let invalid_toml = "volume = [not-valid-toml";
        std::fs::write(&invalid, invalid_toml).unwrap();
        assert_eq!(Config::load_at(&invalid).unwrap(), Config::default());
        assert_eq!(std::fs::read_to_string(&invalid).unwrap(), invalid_toml);
        let _ = std::fs::remove_file(invalid);
    }

    #[test]
    fn load_and_save_use_the_given_path() {
        let path = temporary_config_path();
        let config = Config {
            volume: 0.62,
            current_page: 3,
            ..Config::default()
        };

        config.save_at(&path).unwrap();

        assert_eq!(Config::load_at(&path).unwrap(), config);
        let _ = std::fs::remove_file(path);
    }
}
