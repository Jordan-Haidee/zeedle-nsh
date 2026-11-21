use std::path::Path;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use nsh_core::prelude::*;
use serde::{Deserialize, Serialize};

mod config;
mod logger;
mod spectrum;

mod audio;
mod library;

#[native_module("zeedle-nsh/native")]
fn native(registry: &mut Registry) {
    logger::init_default_logger(None);
    registry.module::<Music>();
    registry.module::<AppConfig>();
}

/// Persisted app configuration exposed to the TSX interface.
#[module]
pub struct AppConfig;

#[module]
impl AppConfig {
    /// Load the TOML configuration as JSON for the TSX UI.
    pub fn load() -> String {
        serde_json::to_string(&config::Config::load()).unwrap_or_else(|_| "{}".into())
    }

    /// Save a complete Config snapshot serialized as JSON.
    pub fn save(value: String) -> String {
        let Ok(config) = serde_json::from_str::<config::Config>(&value) else {
            return "invalid app configuration JSON".into();
        };
        config
            .save()
            .map_or_else(|error| error.to_string(), |_| String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SongInfo {
    pub id: i32,
    pub song_name: String,
    pub singer: String,
    pub duration: String,
    pub duration_secs: f32,
    pub song_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LyricItem {
    pub time: f32,
    pub duration: f32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackState {
    pub position: f32,
    pub duration: f32,
    pub paused: bool,
    pub volume: f32,
    pub error: String,
    pub spectrum: Vec<f32>,
}

/// Rust-backed music library, tag reader and rodio audio engine.
#[module]
pub struct Music;

#[module]
impl Music {
    /// Whether a URL may be handed to the system browser.
    pub fn is_browser_url(url: String) -> bool {
        url.starts_with("https://") || url.starts_with("http://")
    }

    /// Open an external HTTP URL with the user's default browser.
    pub fn open_url(url: String) -> String {
        if !Self::is_browser_url(url.clone()) {
            return "Only HTTP and HTTPS links can be opened".into();
        }
        open::that(url).map_or_else(|error| error.to_string(), |_| String::new())
    }

    /// Set an explicit playback state and acknowledge it after the audio worker applies it.
    pub async fn set_paused(paused: bool) -> bool {
        audio::set_paused(paused).await
    }
    /// Color the original Lucide SVG before decoding, avoiding native bitmap tint transforms.
    pub fn transport_icon(name: String, color: String) -> String {
        let svg = match name.as_str() {
            "play" => include_str!("../assets/icons/play.svg"),
            "pause" => include_str!("../assets/icons/pause.svg"),
            "skipBack" => include_str!("../assets/icons/skip-back.svg"),
            "skipForward" => include_str!("../assets/icons/skip-forward.svg"),
            _ => return String::new(),
        };
        format!(
            "data:image/svg+xml;base64,{}",
            STANDARD.encode(svg.replace("currentColor", &color))
        )
    }
    /// Scan one directory for supported music files and read their Lofty tags.
    pub async fn scan(directory: String) -> String {
        log::info!("scanning music directory: {directory}");
        let songs = library::scan(Path::new(&directory));
        if songs.is_empty() {
            log::warn!("no supported songs found in {directory}");
        } else {
            log::info!("found {} songs in {directory}", songs.len());
        }
        serde_json::to_string(&songs).unwrap_or_else(|error| {
            log::error!("failed to serialize scanned songs: {error}");
            "[]".into()
        })
    }

    /// Read embedded lyrics from an audio file and parse LRC timestamps.
    pub async fn lyrics(path: String) -> String {
        serde_json::to_string(&library::read_lyrics(Path::new(&path)))
            .unwrap_or_else(|_| "[]".into())
    }

    /// Return the embedded album art as a PNG data URL, or an empty string.
    pub async fn cover(path: String) -> String {
        library::cover_data_url(Path::new(&path))
    }

    /// Read the editable text tags from an audio file as JSON.
    pub async fn editor_metadata(path: String) -> String {
        library::editor_metadata(Path::new(&path))
    }

    /// Update the title, artist and album tags with Lofty.
    pub async fn update_metadata(
        path: String,
        title: String,
        artist: String,
        album: String,
        cover_path: String,
        lyrics_path: String,
    ) -> String {
        log::info!("updating metadata for {path}");
        library::update_metadata(
            Path::new(&path),
            &title,
            &artist,
            &album,
            Path::new(&cover_path),
            Path::new(&lyrics_path),
        )
    }

    /// Show a native picker for an embedded cover upload.
    pub async fn choose_cover() -> String {
        #[cfg(not(target_arch = "wasm32"))]
        {
            rfd::FileDialog::new()
                .set_title("Choose cover image")
                .add_filter("Cover images", &["png", "jpg", "jpeg"])
                .pick_file()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        }
        #[cfg(target_arch = "wasm32")]
        {
            String::new()
        }
    }

    /// Show a native picker for an LRC upload.
    pub async fn choose_lyrics() -> String {
        #[cfg(not(target_arch = "wasm32"))]
        {
            rfd::FileDialog::new()
                .set_title("Choose lyrics file")
                .add_filter("LRC lyrics", &["lrc"])
                .pick_file()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        }
        #[cfg(target_arch = "wasm32")]
        {
            String::new()
        }
    }

    /// Preview a local PNG or JPEG file as an image data URL.
    pub async fn cover_file(path: String) -> String {
        let Ok(bytes) = std::fs::read(path) else {
            return String::new();
        };
        library::image_data_url(&bytes).unwrap_or_default()
    }

    /// Read and validate an external LRC file for the metadata editor.
    pub async fn lyrics_file(path: String) -> String {
        library::read_external_lyrics(Path::new(&path)).unwrap_or_default()
    }

    /// Delete a selected audio file. An empty result means it was deleted.
    pub async fn delete_song(path: String) -> String {
        log::info!("deleting song file: {path}");
        std::fs::remove_file(&path).map_or_else(
            |error| {
                log::error!("failed to delete {path}: {error}");
                error.to_string()
            },
            |_| String::new(),
        )
    }

    /// Show a native folder picker. An empty result means it was cancelled.
    pub async fn choose_directory() -> String {
        #[cfg(not(target_arch = "wasm32"))]
        {
            rfd::FileDialog::new()
                .set_title("Choose music directory")
                .pick_folder()
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default()
        }
        #[cfg(target_arch = "wasm32")]
        {
            String::new()
        }
    }

    /// The default Music directory for this user.
    pub fn default_directory() -> String {
        library::default_music_dir()
    }

    /// Start or replace the current track with rodio.
    pub fn play(path: String) {
        log::info!("request to play: {path}");
        audio::play(path, true);
    }

    /// Load the selected track without starting playback.
    pub fn load(path: String) {
        log::info!("request to load: {path}");
        audio::play(path, false);
    }

    /// Toggle playback, starting the selected track when possible.
    pub fn toggle() {
        log::info!("request to toggle playback");
        audio::toggle();
    }

    /// Stop playback and clear the active source.
    pub fn stop() {
        audio::stop();
    }

    /// Seek to a position in seconds.
    pub fn seek(seconds: f32) {
        log::debug!("request to seek to {seconds}s");
        audio::seek(seconds);
    }

    /// Change playback volume between zero and one.
    pub fn set_volume(volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        log::info!("volume changed to {volume}");
        audio::set_volume(volume);
    }

    /// Enable or disable the FFT spectrum view.
    pub fn set_spectrum_enabled(enabled: bool) {
        log::info!("spectrum display set to {enabled}");
        audio::set_spectrum_enabled(enabled);
    }

    /// Current rodio position, duration, pause state, volume and last error.
    pub fn state() -> String {
        audio::state()
    }
}
