use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SongInfo {
    pub id: i32,
    pub song_name: String,
    pub singer: String,
    pub duration: String,
    pub duration_secs: f32,
    pub song_path: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LyricItem {
    pub time: f32,
    pub duration: f32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PlaybackState {
    pub position: f32,
    pub duration: f32,
    pub paused: bool,
    pub volume: f32,
    pub error: String,
    pub spectrum: Vec<f32>,
}

pub fn time_label(seconds: f32) -> String {
    let value = seconds.max(0.0).floor() as u64;
    format!("{:02}:{:02}", value / 60, value % 60)
}

pub fn search_songs(songs: &[SongInfo], query: &str) -> Vec<SongInfo> {
    let term = query.trim().to_lowercase();
    if term.is_empty() {
        return Vec::new();
    }
    let mut matches = songs
        .iter()
        .enumerate()
        .filter_map(|(index, song)| {
            let title = song.song_name.to_lowercase();
            let artist = song.singer.to_lowercase();
            let score = (if title == term {
                100
            } else if title.starts_with(&term) {
                80
            } else if title.contains(&term) {
                60
            } else {
                0
            }) + (if artist == term {
                50
            } else if artist.starts_with(&term) {
                40
            } else if artist.contains(&term) {
                20
            } else {
                0
            });
            (score > 0).then_some((score, index, song.clone()))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    matches.into_iter().map(|(_, _, song)| song).collect()
}

pub fn sorted_songs(songs: &[SongInfo], field: &str, ascending: bool) -> Vec<SongInfo> {
    let mut sorted = songs.to_vec();
    sorted.sort_by(|left, right| {
        let order = match field {
            "singer" => left.singer.to_lowercase().cmp(&right.singer.to_lowercase()),
            "duration_secs" => left
                .duration_secs
                .partial_cmp(&right.duration_secs)
                .unwrap_or(std::cmp::Ordering::Equal),
            _ => left
                .song_name
                .to_lowercase()
                .cmp(&right.song_name.to_lowercase()),
        };
        if ascending { order } else { order.reverse() }
    });
    sorted
}
