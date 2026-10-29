use std::{
    fs::File,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFileExt},
    picture::{MimeType, Picture, PictureType},
    tag::{Accessor, ItemKey, Tag, TagExt},
};
use nsh_core::prelude::*;
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use serde::{Deserialize, Serialize};

mod spectrum;
#[cfg(windows)]
mod startup;
use spectrum::{SpectrumChunk, TapSource, default_spectrum, try_start_spectrum_worker};

#[native_module("zeedle-nsh/native")]
fn native(registry: &mut Registry) {
    registry.module::<Music>();
}

/// Connect the packaged launcher's initially hidden native window.
#[cfg(windows)]
pub fn set_startup_window(handle: isize) {
    startup::set_window(handle);
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

enum Command {
    Load(PathBuf, bool),
    Stop,
    Toggle,
    SetPaused(bool, mpsc::SyncSender<bool>),
    Seek(f32),
    Volume(f32),
}

struct Backend {
    tx: mpsc::Sender<Command>,
    state: Arc<Mutex<PlaybackState>>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_enabled: Arc<AtomicBool>,
}

static BACKEND: OnceLock<Backend> = OnceLock::new();

fn backend() -> &'static Backend {
    BACKEND.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(PlaybackState {
            position: 0.0,
            duration: 0.0,
            paused: true,
            volume: 0.8,
            error: String::new(),
            spectrum: default_spectrum(),
        }));
        let spectrum = Arc::new(Mutex::new(default_spectrum()));
        let spectrum_enabled = Arc::new(AtomicBool::new(true));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (fft_tx, fft_rx) = mpsc::sync_channel::<SpectrumChunk>(4);
            try_start_spectrum_worker(fft_rx, Arc::clone(&spectrum), Arc::clone(&spectrum_enabled));
            let thread_state = Arc::clone(&state);
            let thread_spectrum_enabled = Arc::clone(&spectrum_enabled);
            thread::Builder::new()
                .name("zeedle-audio".into())
                .spawn(move || {
                    let mut sink = DeviceSinkBuilder::open_default_sink();
                    if let Ok(sink) = &mut sink {
                        sink.log_on_drop(false);
                    }
                    let player = sink
                        .as_ref()
                        .ok()
                        .map(|sink| Player::connect_new(sink.mixer()));
                    if let Err(error) = &sink {
                        thread_state.lock().unwrap().error =
                            format!("Audio output unavailable: {error}");
                    }
                    let mut active_duration = 0.0;
                    let mut last_update = Instant::now();

                    loop {
                        match rx.recv_timeout(Duration::from_millis(60)) {
                            Ok(Command::Load(path, autoplay)) => {
                                let result = File::open(&path)
                                    .map_err(|error| error.to_string())
                                    .and_then(|file| {
                                        Decoder::try_from(file).map_err(|error| error.to_string())
                                    });
                                match (player.as_ref(), result) {
                                    (Some(player), Ok(source)) => {
                                        active_duration = source
                                            .total_duration()
                                            .map_or(0.0, |d| d.as_secs_f32());
                                        player.clear();
                                        player.pause();
                                        player.append(TapSource::new(
                                            source,
                                            fft_tx.clone(),
                                            Arc::clone(&thread_spectrum_enabled),
                                        ));
                                        player.set_volume(thread_state.lock().unwrap().volume);
                                        if autoplay {
                                            player.play();
                                        }
                                        let mut state = thread_state.lock().unwrap();
                                        state.position = 0.0;
                                        state.duration = active_duration;
                                        state.paused = !autoplay;
                                        state.error.clear();
                                    }
                                    (_, Err(error)) => thread_state.lock().unwrap().error = error,
                                    (None, Ok(_)) => {}
                                }
                            }
                            Ok(Command::SetPaused(paused, reply)) => {
                                let actual = if let Some(player) = player.as_ref() {
                                    if paused {
                                        player.pause();
                                    } else if !player.empty() {
                                        player.play();
                                    }
                                    player.is_paused()
                                } else {
                                    true
                                };
                                thread_state.lock().unwrap().paused = actual;
                                let _ = reply.send(actual);
                            }
                            Ok(Command::Toggle) => {
                                if let Some(player) = player.as_ref() {
                                    if !player.empty() {
                                        if player.is_paused() {
                                            player.play();
                                        } else {
                                            player.pause();
                                        }
                                    }
                                }
                            }
                            Ok(Command::Stop) => {
                                if let Some(player) = player.as_ref() {
                                    player.clear();
                                }
                                active_duration = 0.0;
                                let mut state = thread_state.lock().unwrap();
                                state.position = 0.0;
                                state.duration = 0.0;
                                state.paused = true;
                            }
                            Ok(Command::Seek(seconds)) => {
                                if let Some(player) = player.as_ref() {
                                    if !player.empty() {
                                        if let Err(error) = player
                                            .try_seek(Duration::from_secs_f32(seconds.max(0.0)))
                                        {
                                            thread_state.lock().unwrap().error = error.to_string();
                                        }
                                    }
                                }
                            }
                            Ok(Command::Volume(volume)) => {
                                if let Some(player) = player.as_ref() {
                                    player.set_volume(volume);
                                }
                                thread_state.lock().unwrap().volume = volume;
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                        }

                        if last_update.elapsed() >= Duration::from_millis(80) {
                            if let Some(player) = player.as_ref() {
                                let mut state = thread_state.lock().unwrap();
                                if !player.empty() {
                                    state.position = player.get_pos().as_secs_f32();
                                    state.paused = player.is_paused();
                                } else if active_duration > 0.0 {
                                    state.position = active_duration;
                                    state.paused = true;
                                }
                            }
                            last_update = Instant::now();
                        }
                    }
                })
                .expect("could not start audio worker");
        }
        #[cfg(target_arch = "wasm32")]
        drop(rx);
        Backend {
            tx,
            state,
            spectrum,
            spectrum_enabled,
        }
    })
}

fn meta(path: &Path) -> Option<SongInfo> {
    let tagged = lofty::read_from_path(path).ok()?;
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");
    let tag = tagged.primary_tag();
    let title = tag
        .and_then(|tag| tag.title())
        .map_or_else(|| file_stem.to_owned(), |value| value.into_owned());
    let artist = tag
        .and_then(|tag| tag.artist())
        .map_or_else(|| "Unknown artist".to_owned(), |value| value.into_owned());
    let duration_secs = tagged.properties().duration().as_secs_f32();
    Some(SongInfo {
        id: 0,
        song_name: title,
        singer: artist,
        duration: format!(
            "{:02}:{:02}",
            duration_secs as u32 / 60,
            duration_secs as u32 % 60
        ),
        duration_secs,
        song_path: path.to_string_lossy().into_owned(),
    })
}

fn scan(path: &Path) -> Vec<SongInfo> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut songs: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "mp3" | "flac" | "wav" | "ogg"
                    )
                })
        })
        .filter_map(|path| meta(&path))
        .collect();
    songs.sort_by_cached_key(|song| song.song_name.to_lowercase());
    for (index, song) in songs.iter_mut().enumerate() {
        song.id = index as i32;
    }
    songs
}

fn read_lyrics(path: &Path) -> Vec<LyricItem> {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return Vec::new();
    };
    let Some(text) = tagged
        .primary_tag()
        .and_then(|tag| {
            tag.get(ItemKey::UnsyncLyrics)
                .or_else(|| tag.get(ItemKey::Lyrics))
        })
        .and_then(|item| item.value().text())
    else {
        return Vec::new();
    };
    let mut lines: Vec<_> = text
        .trim_start_matches('\u{feff}')
        .lines()
        .filter_map(|line| {
            let (stamp, text) = line.split_once(']')?;
            let stamp = stamp.trim_start_matches('[');
            let time = stamp
                .split(':')
                .map(|part| part.parse::<f32>().ok())
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .rev()
                .reduce(|acc, part| acc + part * 60.0)?;
            (time >= 0.0 && !text.trim().is_empty()).then(|| LyricItem {
                time,
                duration: 0.0,
                text: text.trim().to_owned(),
            })
        })
        .collect();
    lines.sort_by(|a, b| a.time.total_cmp(&b.time));
    for index in 0..lines.len().saturating_sub(1) {
        lines[index].duration = lines[index + 1].time - lines[index].time;
    }
    if let Some(last) = lines.last_mut() {
        last.duration = 100.0;
    }
    lines
}

fn cover_data_url(path: &Path) -> String {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return String::new();
    };
    let Some(bytes) = tagged
        .primary_tag()
        .and_then(|tag| {
            tag.pictures().iter().find(|picture| {
                matches!(
                    picture.pic_type(),
                    PictureType::CoverFront | PictureType::CoverBack
                )
            })
        })
        .map(|picture| picture.data())
    else {
        return String::new();
    };
    let Ok(image) = image::load_from_memory(bytes) else {
        return String::new();
    };
    let mut encoded = Cursor::new(Vec::new());
    if image
        .write_to(&mut encoded, image::ImageFormat::Png)
        .is_err()
    {
        return String::new();
    }
    format!(
        "data:image/png;base64,{}",
        STANDARD.encode(encoded.into_inner())
    )
}

fn image_data_url(bytes: &[u8]) -> Result<String, String> {
    let image = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let mut encoded = Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(encoded.into_inner())
    ))
}

fn load_cover_picture(path: &Path) -> Result<Picture, String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let (expected, mime) = match extension.to_ascii_lowercase().as_str() {
        "png" => (image::ImageFormat::Png, MimeType::Png),
        "jpg" | "jpeg" => (image::ImageFormat::Jpeg, MimeType::Jpeg),
        _ => return Err("Choose a PNG or JPEG cover image".into()),
    };
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    if image::guess_format(&bytes).ok() != Some(expected) {
        return Err("The selected file is not a valid PNG or JPEG image".into());
    }
    image::load_from_memory(&bytes).map_err(|error| error.to_string())?;
    Ok(Picture::unchecked(bytes)
        .pic_type(PictureType::CoverFront)
        .mime_type(mime)
        .build())
}

fn read_external_lyrics(path: &Path) -> Result<String, String> {
    if !path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("lrc"))
    {
        return Err("Choose an LRC lyrics file".into());
    }
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let text = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    let lyrics = text.strip_prefix('\u{feff}').unwrap_or(&text).trim();
    if lyrics.is_empty() {
        return Err("The selected lyrics file is empty".into());
    }
    Ok(lyrics.to_owned())
}

fn default_music_dir() -> String {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Music").to_string_lossy().into_owned()
}

/// Rust-backed music library, tag reader and rodio audio engine.
#[module]
pub struct Music;

#[module]
impl Music {
    /// Set an explicit playback state and acknowledge it after the audio worker applies it.
    pub async fn set_paused(paused: bool) -> bool {
        let (reply, received) = mpsc::sync_channel(1);
        if backend()
            .tx
            .send(Command::SetPaused(paused, reply))
            .is_err()
        {
            return true;
        }
        received.recv().unwrap_or(true)
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
    /// Reveal the centered packaged window once the initial library has loaded.
    pub fn finish_startup() {
        #[cfg(windows)]
        startup::ready();
    }
    /// Scan one directory for supported music files and read their Lofty tags.
    pub async fn scan(directory: String) -> String {
        serde_json::to_string(&scan(Path::new(&directory))).unwrap_or_else(|_| "[]".into())
    }

    /// Read embedded lyrics from an audio file and parse LRC timestamps.
    pub async fn lyrics(path: String) -> String {
        serde_json::to_string(&read_lyrics(Path::new(&path))).unwrap_or_else(|_| "[]".into())
    }

    /// Return the embedded album art as a PNG data URL, or an empty string.
    pub async fn cover(path: String) -> String {
        cover_data_url(Path::new(&path))
    }

    /// Read the editable text tags from an audio file as JSON.
    pub async fn editor_metadata(path: String) -> String {
        let Ok(tagged) = lofty::read_from_path(&path) else {
            return "{}".into();
        };
        let tag = tagged.primary_tag();
        let title = tag
            .and_then(|tag| tag.title())
            .map_or_else(String::new, |value| value.into_owned());
        let artist = tag
            .and_then(|tag| tag.artist())
            .map_or_else(String::new, |value| value.into_owned());
        let album = tag
            .and_then(|tag| tag.album())
            .map_or_else(String::new, |value| value.into_owned());
        let lyrics = tag
            .and_then(|tag| {
                tag.get(ItemKey::UnsyncLyrics)
                    .or_else(|| tag.get(ItemKey::Lyrics))
            })
            .and_then(|item| item.value().text())
            .unwrap_or_default()
            .to_owned();
        let cover = cover_data_url(Path::new(&path));
        serde_json::json!({ "title": title, "artist": artist, "album": album, "lyrics": lyrics, "cover": cover }).to_string()
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
        let cover = if cover_path.is_empty() {
            None
        } else {
            match load_cover_picture(Path::new(&cover_path)) {
                Ok(picture) => Some(picture),
                Err(error) => return error,
            }
        };
        let lyrics = if lyrics_path.is_empty() {
            None
        } else {
            match read_external_lyrics(Path::new(&lyrics_path)) {
                Ok(lyrics) => Some(lyrics),
                Err(error) => return error,
            }
        };
        let Ok(mut tagged) = lofty::read_from_path(&path) else {
            return "Could not read this audio file".into();
        };
        let tag_type = tagged.primary_tag_type();
        if tagged.primary_tag_mut().is_none() {
            tagged.insert_tag(Tag::new(tag_type));
        }
        let Some(tag) = tagged.primary_tag_mut() else {
            return "This audio format has no editable tag".into();
        };
        tag.set_title(title.trim().to_owned());
        tag.set_artist(artist.trim().to_owned());
        if album.trim().is_empty() {
            tag.remove_album();
        } else {
            tag.set_album(album.trim().to_owned());
        }
        if let Some(cover) = cover {
            tag.remove_picture_type(PictureType::CoverFront);
            tag.push_picture(cover);
        }
        if let Some(lyrics) = lyrics
            && !tag.insert_text(ItemKey::UnsyncLyrics, lyrics)
        {
            return "This audio format does not support embedded lyrics".into();
        }
        tag.save_to_path(&path, WriteOptions::default())
            .map_or_else(|error| error.to_string(), |_| String::new())
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
        image_data_url(&bytes).unwrap_or_default()
    }

    /// Read and validate an external LRC file for the metadata editor.
    pub async fn lyrics_file(path: String) -> String {
        read_external_lyrics(Path::new(&path)).unwrap_or_default()
    }

    /// Delete a selected audio file. An empty result means it was deleted.
    pub async fn delete_song(path: String) -> String {
        std::fs::remove_file(path).map_or_else(|error| error.to_string(), |_| String::new())
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
        default_music_dir()
    }

    /// Start or replace the current track with rodio.
    pub fn play(path: String) {
        let _ = backend().tx.send(Command::Load(PathBuf::from(path), true));
    }

    /// Load the selected track without starting playback.
    pub fn load(path: String) {
        let _ = backend().tx.send(Command::Load(PathBuf::from(path), false));
    }

    /// Toggle playback, starting the selected track when possible.
    pub fn toggle() {
        let _ = backend().tx.send(Command::Toggle);
    }

    /// Stop playback and clear the active source.
    pub fn stop() {
        let _ = backend().tx.send(Command::Stop);
    }

    /// Seek to a position in seconds.
    pub fn seek(seconds: f32) {
        let _ = backend().tx.send(Command::Seek(seconds));
    }

    /// Change playback volume between zero and one.
    pub fn set_volume(volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        let _ = backend().tx.send(Command::Volume(volume));
    }

    /// Enable or disable the FFT spectrum view.
    pub fn set_spectrum_enabled(enabled: bool) {
        let backend = backend();
        backend.spectrum_enabled.store(enabled, Ordering::Relaxed);
        if !enabled {
            *backend.spectrum.lock().unwrap() = default_spectrum();
        }
    }

    /// Current rodio position, duration, pause state, volume and last error.
    pub fn state() -> String {
        let backend = backend();
        let mut state = backend.state.lock().unwrap().clone();
        state.spectrum = backend.spectrum.lock().unwrap().clone();
        serde_json::to_string(&state).unwrap_or_else(|_| "{}".into())
    }
}
