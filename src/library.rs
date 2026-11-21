use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use lofty::{
    config::WriteOptions,
    file::{AudioFile, TaggedFileExt},
    picture::{MimeType, Picture, PictureType},
    tag::{Accessor, ItemKey, Tag, TagExt},
};

use crate::{LyricItem, SongInfo};

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

pub(crate) fn scan(path: &Path) -> Vec<SongInfo> {
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

pub(crate) fn read_lyrics(path: &Path) -> Vec<LyricItem> {
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

fn cover_picture(tag: &Tag) -> Option<&Picture> {
    tag.pictures()
        .iter()
        .find(|picture| picture.pic_type() == PictureType::CoverFront)
        .or_else(|| {
            tag.pictures()
                .iter()
                .find(|picture| picture.pic_type() == PictureType::CoverBack)
        })
        .or_else(|| tag.pictures().first())
}

pub(crate) fn cover_data_url(path: &Path) -> String {
    let Ok(tagged) = lofty::read_from_path(path) else {
        return String::new();
    };
    let Some(picture) = tagged
        .primary_tag()
        .and_then(cover_picture)
        .or_else(|| tagged.tags().iter().find_map(cover_picture))
    else {
        return String::new();
    };
    let bytes = picture.data();
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

pub(crate) fn editor_metadata(path: &Path) -> String {
    let Ok(tagged) = lofty::read_from_path(path) else {
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
    let cover = cover_data_url(path);
    serde_json::json!({ "title": title, "artist": artist, "album": album, "lyrics": lyrics, "cover": cover }).to_string()
}

pub(crate) fn update_metadata(
    path: &Path,
    title: &str,
    artist: &str,
    album: &str,
    cover_path: &Path,
    lyrics_path: &Path,
) -> String {
    let cover = if cover_path.as_os_str().is_empty() {
        None
    } else {
        match load_cover_picture(cover_path) {
            Ok(picture) => Some(picture),
            Err(error) => return error,
        }
    };
    let lyrics = if lyrics_path.as_os_str().is_empty() {
        None
    } else {
        match read_external_lyrics(lyrics_path) {
            Ok(lyrics) => Some(lyrics),
            Err(error) => return error,
        }
    };
    let Ok(mut tagged) = lofty::read_from_path(path) else {
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
    tag.save_to_path(path, WriteOptions::default())
        .map_or_else(|error| error.to_string(), |_| String::new())
}

pub(crate) fn image_data_url(bytes: &[u8]) -> Result<String, String> {
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

pub(crate) fn load_cover_picture(path: &Path) -> Result<Picture, String> {
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

pub(crate) fn read_external_lyrics(path: &Path) -> Result<String, String> {
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

pub(crate) fn default_music_dir() -> String {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join("Music").to_string_lossy().into_owned()
}
