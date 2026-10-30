use std::time::Duration;

use nsh_base::{input, theme};
use nsh_core::{
    LayoutAnimation, LayoutCurve, LayoutTransition,
    desktop::TitleBarStyle,
    element::{IntoElement, desktop},
    platform::Appearance,
    prelude::*,
    storage::Storage,
};

use crate::Music;

mod model;
use model::{LyricItem, PlaybackState, SongInfo, search_songs, sorted_songs, time_label};

#[derive(Clone, Copy)]
struct UiState {
    page: Signal<u8>,
    directory: Signal<String>,
    songs: Signal<Vec<SongInfo>>,
    current: Signal<Option<SongInfo>>,
    lyrics: Signal<Vec<LyricItem>>,
    art: Signal<String>,
    query: Signal<String>,
    search_results: Signal<Vec<SongInfo>>,
    loading: Signal<bool>,
    sort_field: Signal<String>,
    ascending: Signal<bool>,
    mode: Signal<String>,
    position: Signal<f32>,
    duration: Signal<f32>,
    paused: Signal<bool>,
    volume: Signal<f32>,
    show_spectrum: Signal<bool>,
    spectrum: Signal<Vec<f32>>,
    error: Signal<String>,
    history: Signal<Vec<String>>,
    history_index: Signal<i32>,
    handled_end: Signal<bool>,
    language: Signal<String>,
    follow_system: Signal<bool>,
    light: Signal<bool>,
    selected_action: Signal<Option<SongInfo>>,
    action_position: Signal<Point<Points>>,
    editing_song: Signal<Option<SongInfo>>,
    edit_title: Signal<String>,
    edit_artist: Signal<String>,
    edit_album: Signal<String>,
    edit_cover_path: Signal<String>,
    edit_cover_name: Signal<String>,
    edit_cover_preview: Signal<String>,
    edit_lyrics_path: Signal<String>,
    edit_lyrics_name: Signal<String>,
    edit_lyrics_preview: Signal<String>,
    edit_error: Signal<String>,
    edit_saving: Signal<bool>,
    delete_target: Signal<Option<SongInfo>>,
    lyric_scroll: ElementRef,
    last_lyric_index: Signal<i32>,
    manual_lyrics_until: Signal<u64>,
    ignore_programmatic_scroll_until: Signal<u64>,
    track_width: Signal<f32>,
    lyric_panel_height: Signal<f32>,
}

const COVER: nsh_core::element::FileAsset = asset!("cover.svg");

fn color_token(name: &'static str) -> AdaptiveColor {
    theme().color(name).unwrap_or(theme().foreground)
}

fn localized(language: &str, text: &str) -> String {
    if language == "en" {
        return text.to_string();
    }
    let translated = match (language, text) {
        ("zh", "Gallery") => "曲库",
        ("es", "Gallery") => "Biblioteca",
        ("fr", "Gallery") => "Bibliothèque",
        ("de", "Gallery") => "Mediathek",
        ("ru", "Gallery") => "Библиотека",
        ("zh", "Search") => "搜索",
        ("es", "Search") => "Buscar",
        ("fr", "Search") => "Rechercher",
        ("de", "Search") => "Suche",
        ("ru", "Search") => "Поиск",
        ("zh", "Settings") => "设置",
        ("es", "Settings") => "Ajustes",
        ("fr", "Settings") => "Paramètres",
        ("de", "Settings") => "Einstellungen",
        ("ru", "Settings") => "Настройки",
        ("zh", "About") => "关于",
        ("es", "About") => "Acerca de",
        ("fr", "About") => "À propos",
        ("de", "About") => "Über",
        ("ru", "About") => "О программе",
        ("zh", "Title") => "标题",
        ("es", "Title") => "Título",
        ("fr", "Title") => "Titre",
        ("de", "Title") => "Titel",
        ("ru", "Title") => "Название",
        ("zh", "Artist") => "艺术家",
        ("es", "Artist") => "Artista",
        ("fr", "Artist") => "Artiste",
        ("de", "Artist") => "Künstler",
        ("ru", "Artist") => "Исполнитель",
        ("zh", "Unknown") => "未知",
        ("es", "Unknown") => "Desconocido",
        ("fr", "Unknown") => "Inconnu",
        ("de", "Unknown") => "Unbekannt",
        ("ru", "Unknown") => "Неизвестно",
        ("zh", "No song") => "无歌曲",
        ("es", "No song") => "Sin canción",
        ("fr", "No song") => "Aucun titre",
        ("de", "No song") => "Kein Titel",
        ("ru", "No song") => "Нет песни",
        ("zh", "Artist:") => "艺术家：",
        ("es", "Artist:") => "Artista:",
        ("fr", "Artist:") => "Artiste :",
        ("de", "Artist:") => "Künstler:",
        ("ru", "Artist:") => "Исполнитель:",
        ("zh", "Title:") => "歌曲：",
        ("es", "Title:") => "Título:",
        ("fr", "Title:") => "Titre :",
        ("de", "Title:") => "Titel:",
        ("ru", "Title:") => "Название:",
        ("zh", "Duration") => "时长",
        ("es", "Duration") => "Duración",
        ("fr", "Duration") => "Durée",
        ("de", "Duration") => "Dauer",
        ("ru", "Duration") => "Длительность",
        ("zh", "Search songs or artists...") => "搜索歌曲或艺术家…",
        ("es", "Search songs or artists...") => "Buscar canciones o artistas…",
        ("fr", "Search songs or artists...") => "Rechercher des titres ou artistes…",
        ("de", "Search songs or artists...") => "Titel oder Künstler suchen…",
        ("ru", "Search songs or artists...") => "Поиск песен или исполнителей…",
        ("zh", "Type to search your music") => "输入关键词搜索音乐",
        ("es", "Type to search your music") => "Escribe para buscar música",
        ("fr", "Type to search your music") => "Saisissez pour rechercher",
        ("de", "Type to search your music") => "Musik zum Suchen eingeben",
        ("ru", "Type to search your music") => "Введите запрос для поиска музыки",
        ("zh", "No songs found in this folder") => "此文件夹中没有歌曲",
        ("es", "No songs found in this folder") => "No hay canciones en esta carpeta",
        ("fr", "No songs found in this folder") => "Aucun titre dans ce dossier",
        ("de", "No songs found in this folder") => "Keine Titel in diesem Ordner",
        ("ru", "No songs found in this folder") => "В этой папке нет песен",
        ("zh", "Reading your music library…") => "正在读取音乐库…",
        ("es", "Reading your music library…") => "Leyendo tu biblioteca…",
        ("fr", "Reading your music library…") => "Lecture de la bibliothèque…",
        ("de", "Reading your music library…") => "Musikbibliothek wird gelesen…",
        ("ru", "Reading your music library…") => "Чтение фонотеки…",
        ("zh", "Choose a folder with MP3, FLAC, WAV or OGG files") => {
            "选择包含 MP3、FLAC、WAV 或 OGG 文件的文件夹"
        }
        ("es", "Choose a folder with MP3, FLAC, WAV or OGG files") => {
            "Elige una carpeta con archivos MP3, FLAC, WAV u OGG"
        }
        ("fr", "Choose a folder with MP3, FLAC, WAV or OGG files") => {
            "Choisissez un dossier contenant des fichiers MP3, FLAC, WAV ou OGG"
        }
        ("de", "Choose a folder with MP3, FLAC, WAV or OGG files") => {
            "Ordner mit MP3-, FLAC-, WAV- oder OGG-Dateien wählen"
        }
        ("ru", "Choose a folder with MP3, FLAC, WAV or OGG files") => {
            "Выберите папку с файлами MP3, FLAC, WAV или OGG"
        }
        ("zh", "Playback") => "播放",
        ("es", "Playback") => "Reproducción",
        ("fr", "Playback") => "Lecture",
        ("de", "Playback") => "Wiedergabe",
        ("ru", "Playback") => "Воспроизведение",
        ("zh", "Music directory") => "音乐文件夹",
        ("es", "Music directory") => "Carpeta de música",
        ("fr", "Music directory") => "Dossier musical",
        ("de", "Music directory") => "Musikordner",
        ("ru", "Music directory") => "Папка с музыкой",
        ("zh", "Browse…") => "浏览…",
        ("es", "Browse…") => "Examinar…",
        ("fr", "Browse…") => "Parcourir…",
        ("de", "Browse…") => "Durchsuchen…",
        ("ru", "Browse…") => "Обзор…",
        ("zh", "Spectrum display") => "频谱显示",
        ("es", "Spectrum display") => "Mostrar espectro",
        ("fr", "Spectrum display") => "Afficher le spectre",
        ("de", "Spectrum display") => "Spektrum anzeigen",
        ("ru", "Spectrum display") => "Показывать спектр",
        ("zh", "Appearance") => "外观",
        ("es", "Appearance") => "Apariencia",
        ("fr", "Appearance") => "Apparence",
        ("de", "Appearance") => "Darstellung",
        ("ru", "Appearance") => "Внешний вид",
        ("zh", "Follow system theme") => "跟随系统主题",
        ("es", "Follow system theme") => "Seguir tema del sistema",
        ("fr", "Follow system theme") => "Suivre le thème système",
        ("de", "Follow system theme") => "Systemdesign folgen",
        ("ru", "Follow system theme") => "Следовать теме системы",
        ("zh", "Language") => "语言",
        ("es", "Language") => "Idioma",
        ("fr", "Language") => "Langue",
        ("de", "Language") => "Sprache",
        ("ru", "Language") => "Язык",
        ("zh", "result(s) found") => "个结果",
        ("es", "result(s) found") => "resultado(s)",
        ("fr", "result(s) found") => "résultat(s)",
        ("de", "result(s) found") => "Ergebnis(se)",
        ("ru", "result(s) found") => "результат(ов)",
        ("zh", "No song selected") => "尚未选择歌曲",
        ("es", "No song selected") => "Ninguna canción seleccionada",
        ("fr", "No song selected") => "Aucun titre sélectionné",
        ("de", "No song selected") => "Kein Titel ausgewählt",
        ("ru", "No song selected") => "Песня не выбрана",
        ("zh", "Edit metadata…") => "编辑元数据…",
        ("es", "Edit metadata…") => "Editar metadatos…",
        ("fr", "Edit metadata…") => "Modifier les métadonnées…",
        ("de", "Edit metadata…") => "Metadaten bearbeiten…",
        ("ru", "Edit metadata…") => "Изменить метаданные…",
        ("zh", "Delete song…") => "删除歌曲…",
        ("es", "Delete song…") => "Eliminar canción…",
        ("fr", "Delete song…") => "Supprimer le titre…",
        ("de", "Delete song…") => "Titel löschen…",
        ("ru", "Delete song…") => "Удалить песню…",
        ("zh", "Edit song metadata") => "编辑歌曲元数据",
        ("es", "Edit song metadata") => "Editar metadatos",
        ("fr", "Edit song metadata") => "Modifier les métadonnées",
        ("de", "Edit song metadata") => "Song-Metadaten bearbeiten",
        ("ru", "Edit song metadata") => "Изменить метаданные песни",
        ("zh", "Song title") => "歌曲名",
        ("es", "Song title") => "Título",
        ("fr", "Song title") => "Titre",
        ("de", "Song title") => "Songtitel",
        ("ru", "Song title") => "Название песни",
        ("zh", "Album") => "专辑",
        ("es", "Album") => "Álbum",
        ("fr", "Album") => "Album",
        ("de", "Album") => "Album",
        ("ru", "Album") => "Альбом",
        ("zh", "Album cover") => "专辑封面",
        ("es", "Album cover") => "Portada",
        ("fr", "Album cover") => "Pochette",
        ("de", "Album cover") => "Albumcover",
        ("ru", "Album cover") => "Обложка альбома",
        ("zh", "Lyrics") => "歌词",
        ("es", "Lyrics") => "Letras",
        ("fr", "Lyrics") => "Paroles",
        ("de", "Lyrics") => "Songtext",
        ("ru", "Lyrics") => "Текст песни",
        ("zh", "Select cover image") => "选择封面图片",
        ("es", "Select cover image") => "Elegir portada",
        ("fr", "Select cover image") => "Choisir une pochette",
        ("de", "Select cover image") => "Coverbild auswählen",
        ("ru", "Select cover image") => "Выбрать обложку",
        ("zh", "Select LRC file") => "选择 LRC 歌词文件",
        ("es", "Select LRC file") => "Elegir archivo LRC",
        ("fr", "Select LRC file") => "Choisir un fichier LRC",
        ("de", "Select LRC file") => "LRC-Datei auswählen",
        ("ru", "Select LRC file") => "Выбрать файл LRC",
        ("zh", "Cancel") => "取消",
        ("es", "Cancel") => "Cancelar",
        ("fr", "Cancel") => "Annuler",
        ("de", "Cancel") => "Abbrechen",
        ("ru", "Cancel") => "Отмена",
        ("zh", "Save") => "保存",
        ("es", "Save") => "Guardar",
        ("fr", "Save") => "Enregistrer",
        ("de", "Save") => "Speichern",
        ("ru", "Save") => "Сохранить",
        ("zh", "Delete this song?") => "删除这首歌曲？",
        ("es", "Delete this song?") => "¿Eliminar esta canción?",
        ("fr", "Delete this song?") => "Supprimer ce titre ?",
        ("de", "Delete this song?") => "Diesen Titel löschen?",
        ("ru", "Delete this song?") => "Удалить эту песню?",
        ("zh", "Delete song") => "删除歌曲",
        ("es", "Delete song") => "Eliminar canción",
        ("fr", "Delete song") => "Supprimer le titre",
        ("de", "Delete song") => "Titel löschen",
        ("ru", "Delete song") => "Удалить песню",
        ("zh", "Built by Rodio and NSH") => "由 Rodio 和 NSH 构建",
        ("es", "Built by Rodio and NSH") => "Creado con Rodio y NSH",
        ("fr", "Built by Rodio and NSH") => "Créé avec Rodio et NSH",
        ("de", "Built by Rodio and NSH") => "Erstellt mit Rodio und NSH",
        ("ru", "Built by Rodio and NSH") => "Создано с Rodio и NSH",
        ("zh", "Built by") => "构建者",
        ("es", "Built by") => "Creado por",
        ("fr", "Built by") => "Créé par",
        ("de", "Built by") => "Erstellt von",
        ("ru", "Built by") => "Автор сборки",
        ("zh", "and") => "和",
        ("es", "and") => "y",
        ("fr", "and") => "et",
        ("de", "and") => "und",
        ("ru", "and") => "и",
        ("zh", "Music at the speed of a blink, designed for pure simplicity") => {
            "以眨眼般的速度享受音乐，追求纯粹简洁"
        }
        ("es", "Music at the speed of a blink, designed for pure simplicity") => {
            "Música al instante, diseñada para la simplicidad"
        }
        ("fr", "Music at the speed of a blink, designed for pure simplicity") => {
            "La musique en un clin d'œil, toute en simplicité"
        }
        ("de", "Music at the speed of a blink, designed for pure simplicity") => {
            "Musik im Nu, auf pure Einfachheit ausgelegt"
        }
        ("ru", "Music at the speed of a blink, designed for pure simplicity") => {
            "Музыка мгновенно, с акцентом на простоту"
        }
        ("zh", "Author:") => "作者：",
        ("es", "Author:") => "Autor:",
        ("fr", "Author:") => "Auteur :",
        ("de", "Author:") => "Autor:",
        ("ru", "Author:") => "Автор:",
        ("zh", "Version:") => "版本：",
        ("es", "Version:") => "Versión:",
        ("fr", "Version:") => "Version :",
        ("de", "Version:") => "Version:",
        ("ru", "Version:") => "Версия:",
        ("zh", "Github") => "Github",
        ("es", "Github") => "Github",
        ("fr", "Github") => "Github",
        ("de", "Github") => "Github",
        ("ru", "Github") => "Github",
        ("zh", "No embedded lyrics") => "没有内嵌歌词",
        ("es", "No embedded lyrics") => "No hay letras incrustadas",
        ("fr", "No embedded lyrics") => "Aucune parole intégrée",
        ("de", "No embedded lyrics") => "Keine eingebetteten Songtexte",
        ("ru", "No embedded lyrics") => "Встроенный текст отсутствует",
        ("zh", "Existing cover will be kept") => "将保留现有封面",
        ("es", "Existing cover will be kept") => "Se conservará la portada actual",
        ("fr", "Existing cover will be kept") => "La pochette actuelle sera conservée",
        ("de", "Existing cover will be kept") => "Das vorhandene Cover bleibt erhalten",
        ("ru", "Existing cover will be kept") => "Текущая обложка будет сохранена",
        ("zh", "Existing lyrics will be kept") => "将保留现有歌词",
        ("es", "Existing lyrics will be kept") => "Se conservarán las letras actuales",
        ("fr", "Existing lyrics will be kept") => "Les paroles actuelles seront conservées",
        ("de", "Existing lyrics will be kept") => "Der vorhandene Songtext bleibt erhalten",
        ("ru", "Existing lyrics will be kept") => "Текущий текст будет сохранён",
        ("zh", "Title and artist are required.") => "歌曲名和艺术家必填。",
        ("es", "Title and artist are required.") => "El título y el artista son obligatorios.",
        ("fr", "Title and artist are required.") => "Le titre et l'artiste sont obligatoires.",
        ("de", "Title and artist are required.") => "Titel und Künstler sind erforderlich.",
        ("ru", "Title and artist are required.") => "Укажите название и исполнителя.",
        ("zh", "The audio file will be removed from your device.") => "此音频文件将从设备中删除。",
        ("es", "The audio file will be removed from your device.") => {
            "El archivo de audio se eliminará del dispositivo."
        }
        ("fr", "The audio file will be removed from your device.") => {
            "Le fichier audio sera supprimé de votre appareil."
        }
        ("de", "The audio file will be removed from your device.") => {
            "Die Audiodatei wird vom Gerät entfernt."
        }
        ("ru", "The audio file will be removed from your device.") => {
            "Аудиофайл будет удалён с устройства."
        }
        ("zh", "Saving…") => "正在保存…",
        ("es", "Saving…") => "Guardando…",
        ("fr", "Saving…") => "Enregistrement…",
        ("de", "Saving…") => "Speichern…",
        ("ru", "Saving…") => "Сохранение…",
        _ => text,
    };
    translated.to_string()
}

/// Rust implementation of the Zeedle music player interface.
#[component]
pub fn App() -> impl IntoElement {
    let page = signal(0_u8);
    let expanded = signal(false);
    let directory = signal(Music::default_directory());
    let songs = signal(Vec::<SongInfo>::new());
    let current = signal(None::<SongInfo>);
    let lyrics = signal(Vec::<LyricItem>::new());
    let art = signal(COVER.uri.to_string());
    let query = signal(String::new());
    let search_results = signal(Vec::<SongInfo>::new());
    let loading = signal(true);
    let startup_pending = signal(true);
    let sort_field = signal(String::from("song_name"));
    let ascending = signal(true);
    let mode = signal(String::from("order"));
    let position = signal(0.0_f32);
    let duration = signal(0.0_f32);
    let paused = signal(true);
    let volume = signal(0.8_f32);
    let show_spectrum = signal(true);
    let spectrum = signal(vec![0.0_f32; 48]);
    let error = signal(String::new());
    let history = signal(Vec::<String>::new());
    let history_index = signal(-1_i32);
    let handled_end = signal(false);
    let language = signal(String::from("en"));
    let follow_system = signal(true);
    let light = signal(false);
    let selected_action = signal(None::<SongInfo>);
    let action_position = signal(Point {
        x: px(70.0),
        y: px(110.0),
    });
    let editing_song = signal(None::<SongInfo>);
    let edit_title = signal(String::new());
    let edit_artist = signal(String::new());
    let edit_album = signal(String::new());
    let edit_cover_path = signal(String::new());
    let edit_cover_name = signal(String::new());
    let edit_cover_preview = signal(COVER.uri.to_string());
    let edit_lyrics_path = signal(String::new());
    let edit_lyrics_name = signal(String::new());
    let edit_lyrics_preview = signal(String::new());
    let edit_error = signal(String::new());
    let edit_saving = signal(false);
    let delete_target = signal(None::<SongInfo>);
    let lyric_scroll = element_ref();
    let last_lyric_index = signal(-1_i32);
    let manual_lyrics_until = signal(0_u64);
    let ignore_programmatic_scroll_until = signal(0_u64);
    let track_width = signal(1.0_f32);
    let lyric_panel_height = signal(470.0_f32);
    let app = UiState {
        page,
        directory,
        songs,
        current,
        lyrics,
        art,
        query,
        search_results,
        loading,
        sort_field,
        ascending,
        mode,
        position,
        duration,
        paused,
        volume,
        show_spectrum,
        spectrum,
        error,
        history,
        history_index,
        handled_end,
        language,
        follow_system,
        light,
        selected_action,
        action_position,
        editing_song,
        edit_title,
        edit_artist,
        edit_album,
        edit_cover_path,
        edit_cover_name,
        edit_cover_preview,
        edit_lyrics_path,
        edit_lyrics_name,
        edit_lyrics_preview,
        edit_error,
        edit_saving,
        delete_target,
        lyric_scroll,
        last_lyric_index,
        manual_lyrics_until,
        ignore_programmatic_scroll_until,
        track_width,
        lyric_panel_height,
    };

    desktop::set_title_bar(TitleBarStyle::Hidden, Some(px(40.0)));
    let shortcut_options = KeyOptions {
        when: nsh_core::keymap::When::NotEditing,
        ..Default::default()
    };
    on_key_with("space", shortcut_options, move |_| {
        spawn(async move {
            if app.current.get().is_none() {
                let list =
                    sorted_songs(&app.songs.get(), &app.sort_field.get(), app.ascending.get());
                if let Some(song) = list.first() {
                    play_track(song.clone(), true, app);
                }
            } else {
                let next = !app.paused.get();
                app.paused.set(Music::set_paused(next).await);
            }
        });
    });
    on_key_with("left", shortcut_options, move |_| seek_by(app, -15.0));
    on_key_with("right", shortcut_options, move |_| seek_by(app, 15.0));
    on_key_with("up", shortcut_options, move |_| play_previous(app));
    on_key_with("down", shortcut_options, move |_| play_next(app));
    on_key_with("f1", shortcut_options, move |_| app.page.set(0));
    on_key_with("f2", shortcut_options, move |_| app.page.set(1));
    on_key_with("f3", shortcut_options, move |_| app.page.set(2));
    on_key_with("f4", shortcut_options, move |_| app.page.set(3));
    on_key_with("f5", shortcut_options, move |_| app.page.set(4));

    let refresh = move |path: String| {
        loading.set(true);
        error.set(String::new());
        directory.set(path.clone());
        spawn(async move {
            let response = Music::scan(path.clone()).await;
            match serde_json::from_str::<Vec<SongInfo>>(&response) {
                Ok(found) => {
                    songs.set(found.clone());
                    search_results.set(search_songs(&found, &query.get()));
                    let _ = Storage::set_item("zeedle.music-directory", path);
                    if found.is_empty() {
                        Music::stop();
                        current.set(None);
                        lyrics.set(Vec::new());
                        art.set(COVER.uri.to_string());
                        position.set(0.0);
                        duration.set(0.0);
                    } else {
                        let active_path = current.get().map(|song| song.song_path);
                        let selected = found
                            .iter()
                            .find(|song| Some(&song.song_path) == active_path.as_ref())
                            .cloned();
                        if let Some(song) = selected {
                            current.set(Some(song));
                        } else {
                            let first = found[0].clone();
                            current.set(Some(first.clone()));
                            Music::load(first.song_path.clone());
                            position.set(0.0);
                            duration.set(first.duration_secs);
                            let lyric_json = Music::lyrics(first.song_path.clone()).await;
                            let cover_uri = Music::cover(first.song_path.clone()).await;
                            if let Ok(found_lyrics) =
                                serde_json::from_str::<Vec<LyricItem>>(&lyric_json)
                            {
                                lyrics.set(found_lyrics);
                            }
                            art.set(if cover_uri.is_empty() {
                                COVER.uri.to_string()
                            } else {
                                cover_uri
                            });
                        }
                    }
                }
                Err(parse_error) => error.set(parse_error.to_string()),
            }
            loading.set(false);
            if startup_pending.get() {
                startup_pending.set(false);
                Music::finish_startup();
            }
        });
    };

    spawn(async move {
        let saved_directory =
            Storage::get_item("zeedle.music-directory").unwrap_or_else(Music::default_directory);
        if let Some(saved_volume) = Storage::get_item("zeedle.volume")
            && let Ok(parsed) = saved_volume.parse::<f32>()
        {
            volume.set(parsed.clamp(0.0, 1.0));
        }
        if let Some(saved_mode) = Storage::get_item("zeedle.play-mode")
            && ["repeat", "random"].contains(&saved_mode.as_str())
        {
            mode.set(saved_mode);
        }
        if let Some(saved_spectrum) = Storage::get_item("zeedle.show-spectrum") {
            show_spectrum.set(saved_spectrum == "true");
        }
        if let Some(saved_language) = Storage::get_item("zeedle.language") {
            language.set(saved_language);
        }
        if let Some(saved_theme) = Storage::get_item("zeedle.light-theme") {
            follow_system.set(false);
            light.set(saved_theme == "true");
            nsh_core::element::ambient::set_color_scheme(if saved_theme == "true" {
                Appearance::Light
            } else {
                Appearance::Dark
            });
        } else {
            follow_system.set(true);
            light.set(!nsh_core::element::ambient::dark());
            nsh_core::element::ambient::set_color_scheme(Appearance::System);
        }
        Music::set_volume(volume.get());
        Music::set_spectrum_enabled(show_spectrum.get());
        refresh(saved_directory);
    });

    set_interval(Duration::from_millis(120), move || {
        if let Ok(state) = serde_json::from_str::<PlaybackState>(&Music::state()) {
            position.set(state.position);
            if state.duration > 0.0 {
                duration.set(state.duration);
            }
            paused.set(state.paused);
            volume.set(state.volume);
            spectrum.set(state.spectrum);
            if !state.error.is_empty() {
                error.set(state.error);
            }
            if page.get() == 2 && !state.paused {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                if now >= app.manual_lyrics_until.get() {
                    let lines = app.lyrics.get();
                    let active = lines
                        .iter()
                        .enumerate()
                        .take_while(|(_, line)| line.time <= state.position)
                        .map(|(index, _)| index as i32)
                        .last()
                        .unwrap_or(-1);
                    if active >= 0 && active != app.last_lyric_index.get() {
                        app.last_lyric_index.set(active);
                        app.ignore_programmatic_scroll_until.set(now + 350);
                        app.lyric_scroll.scroll_to(
                            Point {
                                x: px(0.0),
                                y: px(((active - 5).max(0) as f32) * 40.0),
                            },
                            false,
                        );
                    }
                }
            }
            if !state.paused
                && state.duration > 0.0
                && state.position >= state.duration - 0.12
                && !handled_end.get()
            {
                handled_end.set(true);
                play_next(app);
            }
        }
    });

    let language = app.language;
    let nav = move |icon: &'static str, label: &'static str, target: u8| {
        let selected = move || page.get() == target;
        view()
            .test_id(match target {
                0 => "nav-0",
                1 => "nav-1",
                2 => "nav-2",
                3 => "nav-3",
                _ => "nav-4",
            })
            .label(label)
            .h(px(38.0))
            .mb(px(5.0))
            .pl(px(14.0))
            .pr(px(10.0))
            .relative()
            .flex_row()
            .items_center()
            .justify_start()
            .gap(move || px(if expanded.get() { 10.0 } else { 0.0 }))
            .rounded(px(8.0))
            .bg(move || {
                if selected() {
                    color_token("hover")
                } else {
                    color_token("panel")
                }
            })
            .hover(|style| style.bg(move || color_token("hover")))
            .active(|style| style.opacity(0.72))
            .label(label)
            .on_press(move || {
                if target == 2 {
                    app.last_lyric_index.set(-1);
                    app.manual_lyrics_until.set(0);
                }
                page.set(target);
            })
            .child(when(selected, move || {
                view()
                    .absolute()
                    .left(px(3.0))
                    .top(px(11.0))
                    .w(px(3.0))
                    .h(px(16.0))
                    .rounded(px(2.0))
                    .bg(move || theme().primary)
            }))
            .child(
                view()
                    .w(px(20.0))
                    .h(px(18.0))
                    .items_center()
                    .justify_center()
                    .child(
                        when(selected, move || svg_icon(icon, 18.0, "primary"))
                            .otherwise(move || svg_icon(icon, 18.0, "text-secondary")),
                    ),
            )
            .child(
                view()
                    .flex_1()
                    .min_w(px(0.0))
                    .opacity(move || if expanded.get() { 1.0_f32 } else { 0.0_f32 })
                    .transition_opacity()
                    .child(
                        text(move || localized(&language.get(), label))
                            .text_size(px(13.0))
                            .font_weight(if selected() {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(move || {
                                if selected() {
                                    theme().foreground
                                } else {
                                    color_token("text-secondary")
                                }
                            }),
                    ),
            )
    };

    let page_view = dynamic(move || match page.get() {
        0 => library_page(app).into_element(),
        1 => search_page(app).into_element(),
        2 => lyrics_page(app).into_element(),
        3 => settings_page(app, refresh).into_element(),
        _ => about_page(app).into_element(),
    });

    view()
        .flex_1()
        .flex_col()
        .bg(move || color_token("page"))
        .border_1()
        .border_color(move || color_token("border"))
        .child(
            view()
                .h(px(40.0))
                .flex_row()
                .items_center()
                .pl(px(10.0))
                .bg(move || color_token("panel"))
                .border_b_1()
                .border_color(move || color_token("border"))
                .window_drag(true)
                .child(view().w(move || desktop::title_bar().left))
                .child(image(COVER.uri).size(px(18.0)))
                .child(
                    text("Zeedle")
                        .ml(px(6.0))
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(view().flex_1())
                .child(
                    view()
                        .w(px(42.0))
                        .h(px(40.0))
                        .items_center()
                        .justify_center()
                        .hover(|style| style.bg(move || color_token("hover")))
                        .on_press(move || {
                            if !follow_system.get() {
                                let next = !light.get();
                                light.set(next);
                                nsh_core::element::ambient::set_color_scheme(if next {
                                    Appearance::Light
                                } else {
                                    Appearance::Dark
                                });
                                let _ = Storage::set_item("zeedle.light-theme", next.to_string());
                            }
                        })
                        .opacity(move || {
                            if follow_system.get() {
                                0.4_f32
                            } else {
                                1.0_f32
                            }
                        })
                        .child(
                            when(nsh_core::element::ambient::dark, || {
                                svg_icon("moon", 18.0, "text-secondary")
                            })
                            .otherwise(|| svg_icon("sun", 18.0, "text-secondary")),
                        ),
                )
                .child(view().w(move || desktop::title_bar().right)),
        )
        .child(
            view()
                .flex_1()
                .flex_row()
                .min_h(px(0.0))
                .child(
                    view()
                        .w(move || px(if expanded.get() { 200.0 } else { 54.0 }))
                        .flex_col()
                        .px(px(6.0))
                        .pt(px(9.0))
                        .pb(px(10.0))
                        .bg(move || color_token("panel"))
                        .border_r_1()
                        .border_color(move || color_token("border"))
                        .child(
                            view()
                                .test_id("sidebar-toggle")
                                .h(px(36.0))
                                .mb(px(10.0))
                                .pl(px(14.0))
                                .pr(px(10.0))
                                .flex_row()
                                .items_center()
                                .justify_start()
                                .gap(px(10.0))
                                .rounded(px(8.0))
                                .label("Toggle navigation")
                                .hover(|style| style.bg(move || color_token("hover")))
                                .on_press(move || {
                                    animate_layout(LayoutAnimation {
                                        create: Some(LayoutTransition::new(
                                            LayoutCurve::EaseOut,
                                            120.0,
                                        )),
                                        update: Some(LayoutTransition::new(
                                            LayoutCurve::EaseOut,
                                            200.0,
                                        )),
                                        delete: Some(LayoutTransition::new(
                                            LayoutCurve::EaseOut,
                                            120.0,
                                        )),
                                    });
                                    expanded.set(!expanded.get());
                                })
                                .child(
                                    view()
                                        .w(px(20.0))
                                        .h(px(18.0))
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            when(
                                                move || expanded.get(),
                                                || {
                                                    svg_icon(
                                                        "panel-left-close",
                                                        18.0,
                                                        "text-secondary",
                                                    )
                                                },
                                            )
                                            .otherwise(|| {
                                                svg_icon("panel-left-open", 18.0, "text-secondary")
                                            }),
                                        ),
                                ),
                        )
                        .child(view().h(px(5.0)))
                        .child(nav("list-music", "Gallery", 0))
                        .child(nav("search", "Search", 1))
                        .child(nav("captions", "Lyrics", 2))
                        .child(view().flex_1())
                        .child(
                            view()
                                .h(px(1.0))
                                .mx(px(1.0))
                                .mb(px(8.0))
                                .bg(move || theme().border),
                        )
                        .child(nav("settings", "Settings", 3))
                        .child(nav("circle-help", "About", 4)),
                )
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .min_w(px(0.0))
                        .min_h(px(0.0))
                        .child(view().flex_1().min_h(px(0.0)).child(page_view))
                        .child(
                            when(move || page.get() == 0, move || player_panel(app))
                                .otherwise(|| view()),
                        ),
                ),
        )
        .child(
            when(
                move || app.show_spectrum.get(),
                move || {
                    view()
                        .absolute()
                        .left(move || px(if expanded.get() { 200.0 } else { 54.0 }))
                        .right(px(0.0))
                        .bottom(px(0.0))
                        .h(px(60.0))
                        .z_index(5)
                        .pointer_events(PointerEvents::None)
                        .items_center()
                        .justify_end()
                        .child(
                            view()
                                .w(percentage(0.5))
                                .h(px(60.0))
                                .flex_row()
                                .items_end()
                                .opacity(0.3)
                                .child(each(
                                    move || (0..48).collect::<Vec<usize>>(),
                                    |index: &usize| *index,
                                    move |index: ReadSignal<usize>, _| {
                                        let index = index.get();
                                        view()
                                            .flex_1()
                                            .h(move || {
                                                px(app
                                                    .spectrum
                                                    .get()
                                                    .get(index)
                                                    .copied()
                                                    .unwrap_or(0.0)
                                                    .clamp(0.0, 1.0)
                                                    * 60.0)
                                            })
                                            .rounded(px(1.0))
                                            .bg(move || theme().primary)
                                    },
                                )),
                        )
                },
            )
            .otherwise(|| view()),
        )
        .child(
            when(
                move || !error.get().is_empty(),
                move || {
                    view()
                        .absolute()
                        .right(px(15.0))
                        .bottom(move || px(if app.page.get() == 0 { 82.0 } else { 12.0 }))
                        .max_w(px(334.0))
                        .p(px(10.0))
                        .rounded(px(7.0))
                        .border_1()
                        .border_color(move || theme().destructive)
                        .bg(move || color_token("danger-surface"))
                        .on_press(move || error.set(String::new()))
                        .child(
                            text(move || error.get())
                                .text_size(px(11.0))
                                .text_color(move || color_token("danger-text")),
                        )
                },
            )
            .otherwise(|| view()),
        )
        .child(
            when(
                move || app.selected_action.get().is_some(),
                move || action_menu(app),
            )
            .otherwise(|| view()),
        )
        .child(
            when(
                move || app.editing_song.get().is_some(),
                move || editor_overlay(app, refresh),
            )
            .otherwise(|| view()),
        )
        .child(
            when(
                move || app.delete_target.get().is_some(),
                move || delete_overlay(app, refresh),
            )
            .otherwise(|| view()),
        )
}

fn svg_icon(name: &'static str, size: f32, tint: &'static str) -> impl IntoElement {
    let source = match name {
        "list-music" => asset!("icons/list-music.svg"),
        "search" => asset!("icons/search.svg"),
        "captions" => asset!("icons/captions.svg"),
        "settings" => asset!("icons/settings.svg"),
        "circle-help" => asset!("icons/circle-help.svg"),
        "sun" => asset!("icons/sun.svg"),
        "moon" => asset!("icons/moon.svg"),
        "panel-left-open" => asset!("icons/panel-left-open.svg"),
        "panel-left-close" => asset!("icons/panel-left-close.svg"),
        "play" => asset!("icons/play.svg"),
        "pause" => asset!("icons/pause.svg"),
        "skip-back" => asset!("icons/skip-back.svg"),
        "skip-forward" => asset!("icons/skip-forward.svg"),
        "shuffle" => asset!("icons/shuffle.svg"),
        "repeat" => asset!("icons/repeat.svg"),
        "repeat-2" => asset!("icons/repeat-2.svg"),
        "volume-2" => asset!("icons/volume-2.svg"),
        "arrow-up" => asset!("icons/arrow-up.svg"),
        "arrow-down" => asset!("icons/arrow-down.svg"),
        "check" => asset!("icons/check.svg"),
        "x" => asset!("icons/x.svg"),
        _ => asset!("icons/search.svg"),
    };
    image(source.uri)
        .size(px(size))
        .tint_color(move || color_token(tint))
}

fn transport_icon(name: &'static str, tint: &'static str) -> impl IntoElement {
    let name = match name {
        "skip-back" => "skipBack",
        "skip-forward" => "skipForward",
        other => other,
    };
    let color = move || nsh_core::element::ambient::color(color_token(tint)).to_hex();
    view()
        .absolute()
        .left(px(5.0))
        .top(px(5.0))
        .size(px(26.0))
        .pointer_events(PointerEvents::None)
        .items_center()
        .justify_center()
        .child(
            image(move || Music::transport_icon(name.to_string(), color()))
                .size(px(26.0))
                .resize_mode(ResizeMode::Contain),
        )
}

fn play_track(song: SongInfo, add_history: bool, app: UiState) {
    app.current.set(Some(song.clone()));
    app.last_lyric_index.set(-1);
    app.manual_lyrics_until.set(0);
    app.position.set(0.0);
    app.duration.set(song.duration_secs);
    app.paused.set(false);
    app.handled_end.set(false);
    if add_history {
        let mut paths = app.history.get()[..(app.history_index.get() + 1).max(0) as usize].to_vec();
        paths.push(song.song_path.clone());
        app.history.set(paths.clone());
        app.history_index.set(paths.len() as i32 - 1);
    }
    Music::play(song.song_path.clone());
    spawn(async move {
        let lyric_json = Music::lyrics(song.song_path.clone()).await;
        let cover_uri = Music::cover(song.song_path.clone()).await;
        if app
            .current
            .get()
            .as_ref()
            .is_some_and(|active| active.song_path == song.song_path)
        {
            if let Ok(found) = serde_json::from_str::<Vec<LyricItem>>(&lyric_json) {
                app.lyrics.set(found);
            }
            app.art.set(if cover_uri.is_empty() {
                COVER.uri.to_string()
            } else {
                cover_uri
            });
        }
    });
}

fn seek_by(app: UiState, seconds: f32) {
    let target = if app.page.get() == 2 && !app.lyrics.get().is_empty() {
        let lines = app.lyrics.get();
        if seconds < 0.0 {
            lines
                .iter()
                .rev()
                .find(|line| line.time < app.position.get() - 0.5)
                .map_or(0.0, |line| line.time)
        } else {
            lines
                .iter()
                .find(|line| line.time > app.position.get() + 0.5)
                .map_or(app.position.get(), |line| line.time)
        }
    } else {
        (app.position.get() + seconds).clamp(0.0, app.duration.get())
    };
    app.position.set(target);
    Music::seek(target);
}

fn play_next(app: UiState) {
    let list = sorted_songs(&app.songs.get(), &app.sort_field.get(), app.ascending.get());
    if list.is_empty() {
        return;
    }
    if app.mode.get() == "repeat" {
        if let Some(song) = app.current.get() {
            play_track(song, false, app);
        }
        return;
    }
    let current_index = list
        .iter()
        .position(|song| {
            app.current
                .get()
                .as_ref()
                .is_some_and(|current| current.song_path == song.song_path)
        })
        .unwrap_or(0);
    let next_index = if app.mode.get() == "random" && list.len() > 1 {
        (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as usize)
            % list.len()
    } else {
        (current_index + 1) % list.len()
    };
    play_track(list[next_index].clone(), true, app);
}

fn play_previous(app: UiState) {
    let previous = app.history_index.get() - 1;
    if previous >= 0 {
        app.history_index.set(previous);
        let path = app.history.get().get(previous as usize).cloned();
        if let Some(song) = app
            .songs
            .get()
            .into_iter()
            .find(|song| Some(song.song_path.clone()) == path)
        {
            play_track(song, false, app);
            return;
        }
    }
    let list = sorted_songs(&app.songs.get(), &app.sort_field.get(), app.ascending.get());
    if list.is_empty() {
        return;
    }
    let index = list
        .iter()
        .position(|song| {
            app.current
                .get()
                .as_ref()
                .is_some_and(|current| current.song_path == song.song_path)
        })
        .unwrap_or(0);
    let previous = (index + list.len() - 1) % list.len();
    play_track(list[previous].clone(), true, app);
}

fn begin_edit(app: UiState, song: SongInfo) {
    app.editing_song.set(Some(song.clone()));
    app.edit_title.set(song.song_name.clone());
    app.edit_artist.set(song.singer.clone());
    app.edit_album.set(String::new());
    app.edit_cover_path.set(String::new());
    app.edit_cover_name.set(String::new());
    app.edit_cover_preview.set(COVER.uri.to_string());
    app.edit_lyrics_path.set(String::new());
    app.edit_lyrics_name.set(String::new());
    app.edit_lyrics_preview.set(String::new());
    app.edit_error.set(String::new());
    spawn(async move {
        let json = Music::editor_metadata(song.song_path.clone()).await;
        if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&json)
            && app
                .editing_song
                .get()
                .as_ref()
                .is_some_and(|active| active.song_path == song.song_path)
        {
            app.edit_title.set(
                metadata["title"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(&song.song_name)
                    .to_string(),
            );
            app.edit_artist.set(
                metadata["artist"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(&song.singer)
                    .to_string(),
            );
            app.edit_album
                .set(metadata["album"].as_str().unwrap_or_default().to_string());
            app.edit_cover_preview.set(
                metadata["cover"]
                    .as_str()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(COVER.uri)
                    .to_string(),
            );
            app.edit_cover_name.set(
                if metadata["cover"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
                {
                    "Embedded artwork"
                } else {
                    "No cover image"
                }
                .to_string(),
            );
            app.edit_lyrics_preview
                .set(metadata["lyrics"].as_str().unwrap_or_default().to_string());
            app.edit_lyrics_name.set(
                if metadata["lyrics"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
                {
                    "Embedded lyrics"
                } else {
                    "No embedded lyrics"
                }
                .to_string(),
            );
        }
    });
}

fn action_menu(app: UiState) -> impl IntoElement {
    let selected = app.selected_action.get();
    let edit_song = selected.clone();
    let delete_song = selected.clone();
    view()
        .test_id("song-action-menu")
        .absolute()
        .left(move || px(app.action_position.get().x.0))
        .top(move || px(app.action_position.get().y.0))
        .z_index(25)
        .p(px(8.0))
        .flex_col()
        .gap(px(4.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(move || color_token("border"))
        .bg(move || color_token("panel"))
        .child(
            text(move || {
                selected
                    .as_ref()
                    .map_or_else(String::new, |song| song.song_name.clone())
            })
            .px(px(8.0))
            .py(px(6.0))
            .text_size(px(12.0))
            .text_color(move || color_token("text-secondary")),
        )
        .child(
            view()
                .px(px(12.0))
                .py(px(7.0))
                .rounded(px(4.0))
                .hover(|style| style.bg(move || color_token("hover")))
                .on_press(move || {
                    app.selected_action.set(None);
                    if let Some(song) = edit_song.clone() {
                        begin_edit(app, song);
                    }
                })
                .child(
                    text(move || localized(&app.language.get(), "Edit metadata…"))
                        .text_size(px(13.0))
                        .text_color(move || theme().foreground),
                ),
        )
        .child(
            view()
                .px(px(12.0))
                .py(px(7.0))
                .rounded(px(4.0))
                .hover(|style| style.bg(move || color_token("hover")))
                .on_press(move || {
                    app.delete_target.set(delete_song.clone());
                    app.selected_action.set(None);
                })
                .child(
                    text(move || localized(&app.language.get(), "Delete song…"))
                        .text_size(px(13.0))
                        .text_color(move || theme().destructive),
                ),
        )
}

fn editor_overlay(app: UiState, refresh: impl Fn(String) + Copy + 'static) -> impl IntoElement {
    let choose_cover = move || {
        spawn(async move {
            let path = Music::choose_cover().await;
            if path.is_empty() {
                return;
            }
            let preview = Music::cover_file(path.clone()).await;
            if preview.is_empty() {
                app.edit_error
                    .set("Could not read that image file.".to_string());
            } else {
                app.edit_cover_name
                    .set(path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string());
                app.edit_cover_path.set(path);
                app.edit_cover_preview.set(preview);
                app.edit_error.set(String::new());
            }
        });
    };
    let choose_lyrics = move || {
        spawn(async move {
            let path = Music::choose_lyrics().await;
            if path.is_empty() {
                return;
            }
            let contents = Music::lyrics_file(path.clone()).await;
            if contents.is_empty() {
                app.edit_error
                    .set("Could not read that LRC file.".to_string());
            } else {
                app.edit_lyrics_name
                    .set(path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string());
                app.edit_lyrics_path.set(path);
                app.edit_lyrics_preview.set(contents);
                app.edit_error.set(String::new());
            }
        });
    };
    let save = move || {
        let Some(song) = app.editing_song.get() else {
            return;
        };
        if app.edit_saving.get() {
            return;
        }
        if app.edit_title.get().trim().is_empty() || app.edit_artist.get().trim().is_empty() {
            app.edit_error.set(localized(
                &app.language.get(),
                "Title and artist are required.",
            ));
            return;
        }
        app.edit_saving.set(true);
        app.edit_error.set(String::new());
        spawn(async move {
            let message = Music::update_metadata(
                song.song_path,
                app.edit_title.get(),
                app.edit_artist.get(),
                app.edit_album.get(),
                app.edit_cover_path.get(),
                app.edit_lyrics_path.get(),
            )
            .await;
            if message.is_empty() {
                app.editing_song.set(None);
                refresh(app.directory.get());
            } else {
                app.edit_error.set(message);
            }
            app.edit_saving.set(false);
        });
    };
    view()
        .absolute()
        .inset_0()
        .z_index(20)
        .items_center()
        .justify_center()
        .p(px(10.0))
        .child(
            view()
                .w(px(520.0))
                .h(px(450.0))
                .max_w(px(720.0))
                .max_h(px(490.0))
                .p(px(16.0))
                .flex_col()
                .gap(px(7.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(move || color_token("border"))
                .bg(move || color_token("page"))
                .child(
                    text(move || localized(&app.language.get(), "Edit song metadata"))
                        .text_size(px(18.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            text(move || localized(&app.language.get(), "Song title"))
                                .text_size(px(14.0))
                                .text_color(move || color_token("text-secondary")),
                        )
                        .child(
                            input()
                                .test_id("edit-title")
                                .value(move || app.edit_title.get())
                                .on_change(move |value: &str| app.edit_title.set(value.to_string()))
                                .h(px(32.0))
                                .px(px(10.0))
                                .rounded(px(4.0))
                                .border_1()
                                .border_color(move || color_token("border"))
                                .bg(move || color_token("raised"))
                                .text_size(px(14.0))
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(12.0))
                        .child(
                            view()
                                .flex_1()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    text(move || localized(&app.language.get(), "Artist"))
                                        .text_size(px(14.0))
                                        .text_color(move || color_token("text-secondary")),
                                )
                                .child(
                                    input()
                                        .test_id("edit-artist")
                                        .value(move || app.edit_artist.get())
                                        .on_change(move |value: &str| {
                                            app.edit_artist.set(value.to_string())
                                        })
                                        .h(px(32.0))
                                        .px(px(10.0))
                                        .rounded(px(4.0))
                                        .border_1()
                                        .border_color(move || color_token("border"))
                                        .bg(move || color_token("raised"))
                                        .text_size(px(14.0))
                                        .text_color(move || theme().foreground),
                                ),
                        )
                        .child(
                            view()
                                .flex_1()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    text(move || localized(&app.language.get(), "Album"))
                                        .text_size(px(14.0))
                                        .text_color(move || color_token("text-secondary")),
                                )
                                .child(
                                    input()
                                        .test_id("edit-album")
                                        .value(move || app.edit_album.get())
                                        .on_change(move |value: &str| {
                                            app.edit_album.set(value.to_string())
                                        })
                                        .h(px(32.0))
                                        .px(px(10.0))
                                        .rounded(px(4.0))
                                        .border_1()
                                        .border_color(move || color_token("border"))
                                        .bg(move || color_token("raised"))
                                        .text_size(px(14.0))
                                        .text_color(move || theme().foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .h(px(200.0))
                        .flex_row()
                        .gap(px(10.0))
                        .child(
                            view()
                                .flex_1()
                                .min_w(px(0.0))
                                .p(px(8.0))
                                .flex_col()
                                .gap(px(6.0))
                                .rounded(px(8.0))
                                .border_1()
                                .border_color(move || color_token("border"))
                                .bg(move || color_token("panel"))
                                .child(
                                    text(move || localized(&app.language.get(), "Album cover"))
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(image(move || app.edit_cover_preview.get()).size(px(100.0)))
                                .child(
                                    view()
                                        .h(px(32.0))
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(4.0))
                                        .bg(move || color_token("raised"))
                                        .on_press(choose_cover)
                                        .child(
                                            text(move || {
                                                localized(&app.language.get(), "Select cover image")
                                            })
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                        ),
                                )
                                .child(
                                    text(move || {
                                        if app.edit_cover_path.get().is_empty() {
                                            localized(
                                                &app.language.get(),
                                                "Existing cover will be kept",
                                            )
                                        } else {
                                            app.edit_cover_name.get()
                                        }
                                    })
                                    .text_size(px(11.0))
                                    .text_color(move || color_token("text-tertiary")),
                                ),
                        )
                        .child(
                            view()
                                .flex_1()
                                .min_w(px(0.0))
                                .p(px(8.0))
                                .flex_col()
                                .gap(px(6.0))
                                .rounded(px(8.0))
                                .border_1()
                                .border_color(move || color_token("border"))
                                .bg(move || color_token("panel"))
                                .child(
                                    text(move || localized(&app.language.get(), "Lyrics"))
                                        .text_size(px(14.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(
                                    scroll()
                                        .h(px(100.0))
                                        .p(px(8.0))
                                        .rounded(px(4.0))
                                        .bg(move || color_token("raised"))
                                        .child(
                                            text(move || {
                                                if app.edit_lyrics_preview.get().is_empty() {
                                                    localized(
                                                        &app.language.get(),
                                                        "No embedded lyrics",
                                                    )
                                                } else {
                                                    app.edit_lyrics_preview.get()
                                                }
                                            })
                                            .text_size(px(11.0))
                                            .text_color(move || theme().foreground),
                                        ),
                                )
                                .child(
                                    view()
                                        .h(px(32.0))
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(4.0))
                                        .bg(move || color_token("raised"))
                                        .on_press(choose_lyrics)
                                        .child(
                                            text(move || {
                                                localized(&app.language.get(), "Select LRC file")
                                            })
                                            .text_size(px(13.0))
                                            .text_color(move || theme().foreground),
                                        ),
                                )
                                .child(
                                    text(move || {
                                        if app.edit_lyrics_path.get().is_empty() {
                                            "Existing lyrics will be kept".to_string()
                                        } else {
                                            app.edit_lyrics_name.get()
                                        }
                                    })
                                    .text_size(px(11.0))
                                    .text_color(move || color_token("text-tertiary")),
                                ),
                        ),
                )
                .child(
                    when(
                        move || !app.edit_error.get().is_empty(),
                        move || {
                            text(move || app.edit_error.get())
                                .text_size(px(11.0))
                                .text_color(move || theme().destructive)
                        },
                    )
                    .otherwise(|| view()),
                )
                .child(view().flex_1())
                .child(view().h(px(1.0)).bg(move || color_token("border")))
                .child(
                    view()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.0))
                        .child(
                            view()
                                .w(px(68.0))
                                .h(px(32.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .bg(move || color_token("raised"))
                                .on_press(move || app.editing_song.set(None))
                                .child(
                                    text(move || localized(&app.language.get(), "Cancel"))
                                        .text_size(px(13.0))
                                        .text_color(move || theme().foreground),
                                ),
                        )
                        .child(
                            view()
                                .w(px(55.0))
                                .h(px(32.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .bg(move || color_token("button"))
                                .opacity(move || {
                                    if app.edit_saving.get() {
                                        0.65_f32
                                    } else {
                                        1.0_f32
                                    }
                                })
                                .on_press(save)
                                .child(
                                    text(move || {
                                        localized(
                                            &app.language.get(),
                                            if app.edit_saving.get() {
                                                "Saving…"
                                            } else {
                                                "Save"
                                            },
                                        )
                                    })
                                    .text_size(px(13.0))
                                    .text_color(move || theme().foreground),
                                ),
                        ),
                ),
        )
}

fn delete_overlay(app: UiState, refresh: impl Fn(String) + Copy + 'static) -> impl IntoElement {
    let confirm = move || {
        let Some(song) = app.delete_target.get() else {
            return;
        };
        app.delete_target.set(None);
        spawn(async move {
            let message = Music::delete_song(song.song_path).await;
            if message.is_empty() {
                refresh(app.directory.get());
            } else {
                app.error.set(message);
            }
        });
    };
    view()
        .absolute()
        .inset_0()
        .z_index(21)
        .items_center()
        .justify_center()
        .p(px(20.0))
        .child(
            view()
                .w(px(370.0))
                .p(px(20.0))
                .flex_col()
                .gap(px(11.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(move || color_token("border"))
                .bg(move || color_token("page"))
                .child(
                    text(move || localized(&app.language.get(), "Delete this song?"))
                        .text_size(px(17.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().foreground),
                )
                .child(
                    text(move || {
                        app.delete_target
                            .get()
                            .map_or_else(String::new, |song| song.song_name)
                    })
                    .text_size(px(13.0))
                    .text_color(move || theme().foreground),
                )
                .child(
                    text(move || {
                        localized(
                            &app.language.get(),
                            "The audio file will be removed from your device.",
                        )
                    })
                    .text_size(px(11.0))
                    .text_color(move || color_token("text-secondary")),
                )
                .child(
                    view()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.0))
                        .mt(px(5.0))
                        .child(
                            view()
                                .px(px(13.0))
                                .py(px(8.0))
                                .rounded(px(4.0))
                                .bg(move || color_token("raised"))
                                .on_press(move || app.delete_target.set(None))
                                .child(
                                    text(move || localized(&app.language.get(), "Cancel"))
                                        .text_size(px(12.0))
                                        .text_color(move || color_token("text-secondary")),
                                ),
                        )
                        .child(
                            view()
                                .px(px(14.0))
                                .py(px(8.0))
                                .rounded(px(4.0))
                                .bg(move || theme().destructive)
                                .on_press(confirm)
                                .child(
                                    text(move || localized(&app.language.get(), "Delete song"))
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || color_token("destructive-foreground")),
                                ),
                        ),
                ),
        )
}

fn song_row(app: UiState, song: SongInfo) -> impl IntoElement {
    let song_path = song.song_path.clone();
    let active = computed(move || {
        app.current
            .get()
            .is_some_and(|current| current.song_path == song_path)
    });
    let click_song = song.clone();
    let action_song = song.clone();
    let context_song = song.clone();
    let title = song.song_name.clone();
    view()
        .test_id(&format!("song-row-{}", song.id))
        .h(px(30.0))
        .flex_row()
        .items_center()
        .justify_between()
        .hover(|style| style.bg(move || color_token("song-hover")))
        .bg(move || {
            if active.get() {
                color_token("hover")
            } else {
                color_token("page")
            }
        })
        .on(EventKind::Press, move |payload: &EventPayload| {
            if let EventPayload::Press(press) = payload
                && press.clicks > 1
            {
                play_track(click_song.clone(), true, app);
            }
        })
        .on_long_press(move || app.selected_action.set(Some(action_song.clone())))
        .on(EventKind::ContextMenu, move |payload: &EventPayload| {
            if let EventPayload::ContextMenu(event) = payload {
                app.action_position.set(event.page_position);
                app.selected_action.set(Some(context_song.clone()));
            }
        })
        .child(view().w(percentage(0.33)).child(dynamic(move || {
            if active.get() {
                text(title.clone())
                    .ml(percentage(0.4))
                    .text_size(px(14.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme().primary)
            } else {
                text(title.clone())
                    .ml(percentage(0.4))
                    .text_size(px(14.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme().foreground)
            }
        })))
        .child(
            view().w(percentage(0.33)).child(
                text(song.singer.clone())
                    .ml(percentage(0.4))
                    .text_size(px(14.0))
                    .text_color(move || {
                        if active.get() {
                            theme().primary
                        } else {
                            theme().foreground
                        }
                    }),
            ),
        )
        .child(
            view().w(percentage(0.33)).child(
                text(song.duration.clone())
                    .ml(percentage(0.4))
                    .text_size(px(14.0))
                    .text_color(move || {
                        if active.get() {
                            theme().primary
                        } else {
                            theme().foreground
                        }
                    }),
            ),
        )
}

fn song_list(app: UiState, songs: Memo<Vec<SongInfo>>) -> impl IntoElement {
    virtual_list(
        move || songs.with(|songs| songs.len()),
        move |position| {
            dynamic(move || {
                songs
                    .with(|songs| songs.get(position.index()).cloned())
                    .map_or_else(
                        || view().into_element(),
                        |song| song_row(app, song).into_element(),
                    )
            })
        },
    )
    .keys(move || {
        songs.with(|songs| {
            songs
                .iter()
                .map(|song| song.song_path.clone())
                .collect::<Vec<_>>()
        })
    })
    .estimated_item_size(px(30.0))
    .flex_1()
    .min_h(px(0.0))
}

fn library_page(app: UiState) -> impl IntoElement {
    let songs = computed(move || {
        sorted_songs(&app.songs.get(), &app.sort_field.get(), app.ascending.get())
    });
    let sort_header = move |label: &'static str, field: &'static str| {
        view()
            .w(percentage(0.33))
            .flex_row()
            .items_center()
            .gap(px(5.0))
            .on_press(move || {
                if app.sort_field.get() == field {
                    app.ascending.set(!app.ascending.get());
                } else {
                    app.sort_field.set(field.to_string());
                    app.ascending.set(true);
                }
            })
            .child(
                text(move || localized(&app.language.get(), label))
                    .ml(percentage(0.4))
                    .text_size(px(14.0))
                    .text_color(move || color_token("text-tertiary")),
            )
            .child(
                when(
                    move || app.sort_field.get() == field,
                    move || {
                        when(
                            move || app.ascending.get(),
                            || svg_icon("arrow-up", 12.0, "text-tertiary"),
                        )
                        .otherwise(|| svg_icon("arrow-down", 12.0, "text-tertiary"))
                    },
                )
                .otherwise(|| view()),
            )
    };
    view()
        .test_id("screen-0")
        .flex_1()
        .flex_col()
        .min_h(px(0.0))
        .bg(move || color_token("page"))
        .child(
            view()
                .h(px(30.0))
                .flex_row()
                .justify_between()
                .child(sort_header("Title", "song_name"))
                .child(sort_header("Artist", "singer"))
                .child(sort_header("Duration", "duration_secs")),
        )
        .child(
            view()
                .flex_1()
                .min_h(px(0.0))
                .relative()
                .flex_col()
                .child(song_list(app, songs))
                .child(
                    when(
                        move || app.loading.get() || songs.with(|songs| songs.is_empty()),
                        move || {
                            view()
                                .absolute()
                                .left(px(0.0))
                                .right(px(0.0))
                                .top(px(0.0))
                                .bottom(px(0.0))
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(8.0))
                                .child(svg_icon("search", 24.0, "text-tertiary"))
                                .child(
                                    text(move || {
                                        localized(
                                            &app.language.get(),
                                            if app.loading.get() {
                                                "Reading your music library…"
                                            } else {
                                                "No songs found in this folder"
                                            },
                                        )
                                    })
                                    .text_size(px(13.0))
                                    .text_color(move || color_token("text-tertiary")),
                                )
                                .child(
                                    text(move || {
                                        localized(
                                            &app.language.get(),
                                            "Choose a folder with MP3, FLAC, WAV or OGG files",
                                        )
                                    })
                                    .text_size(px(12.0))
                                    .text_color(move || color_token("text-tertiary")),
                                )
                        },
                    )
                    .otherwise(|| view()),
                ),
        )
}

fn search_page(app: UiState) -> impl IntoElement {
    let results = computed(move || app.search_results.get());
    let update_query = move |value: &str| {
        app.query.set(value.to_string());
        app.search_results
            .set(search_songs(&app.songs.get(), value));
    };
    view()
        .test_id("screen-1")
        .flex_1()
        .flex_col()
        .min_h(px(0.0))
        .bg(move || color_token("page"))
        .child(
            view()
                .h(px(40.0))
                .px(px(12.0))
                .flex_row()
                .items_center()
                .child(
                    view()
                        .flex_1()
                        .h(px(24.0))
                        .ml(px(3.0))
                        .mr(px(2.0))
                        .flex_row()
                        .items_center()
                        .rounded(px(3.0))
                        .border_b(px(2.0))
                        .border_color(move || theme().primary)
                        .child(
                            input()
                                .test_id("song-search")
                                .value(move || app.query.get())
                                .placeholder(move || {
                                    localized(&app.language.get(), "Search songs or artists...")
                                })
                                .on_change(move |value: &str| update_query(value))
                                .flex_1()
                                .h(px(24.0))
                                .px(px(4.0))
                                .text_size(px(13.0))
                                .text_color(move || theme().foreground)
                                .bg(move || color_token("page")),
                        )
                        .child(
                            when(
                                move || !app.query.get().is_empty(),
                                move || {
                                    view()
                                        .p(px(4.0))
                                        .on_press(move || update_query(""))
                                        .child(svg_icon("x", 14.0, "text-tertiary"))
                                },
                            )
                            .otherwise(|| view()),
                        ),
                ),
        )
        .child(
            when(
                move || !app.query.get().is_empty(),
                move || {
                    view().h(px(26.0)).justify_center().px(px(12.0)).child(
                        text(move || {
                            format!(
                                "{} {}",
                                app.search_results.get().len(),
                                localized(&app.language.get(), "result(s) found")
                            )
                        })
                        .text_size(px(12.0))
                        .text_color(move || color_token("text-tertiary")),
                    )
                },
            )
            .otherwise(|| view()),
        )
        .child(
            when(
                move || !app.query.get().is_empty(),
                move || {
                    view()
                        .flex_1()
                        .min_h(px(0.0))
                        .relative()
                        .flex_col()
                        .child(song_list(app, results))
                        .child(
                            when(
                                move || results.with(|songs| songs.is_empty()),
                                move || {
                                    view()
                                        .absolute()
                                        .left(px(0.0))
                                        .right(px(0.0))
                                        .top(px(0.0))
                                        .bottom(px(0.0))
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            text(move || {
                                                localized(
                                                    &app.language.get(),
                                                    "No songs found in this folder",
                                                )
                                            })
                                            .text_size(px(13.0))
                                            .text_color(move || color_token("text-tertiary")),
                                        )
                                },
                            )
                            .otherwise(|| view()),
                        )
                },
            )
            .otherwise(move || {
                view()
                    .flex_1()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(8.0))
                    .child(svg_icon("search", 44.0, "text-tertiary"))
                    .child(
                        text(move || localized(&app.language.get(), "Type to search your music"))
                            .text_size(px(14.0))
                            .text_color(move || color_token("text-tertiary")),
                    )
            }),
        )
}

fn lyrics_page(app: UiState) -> impl IntoElement {
    view()
        .test_id("screen-2")
        .flex_1()
        .flex_row()
        .min_h(px(0.0))
        .bg(move || color_token("page"))
        .on_layout(move |bounds: &Bounds<Points>| app.lyric_panel_height.set(bounds.size.height.0))
        .child(
            view()
                .w(move || {
                    if app.lyrics.get().is_empty() {
                        percentage(1.0)
                    } else {
                        percentage(0.3)
                    }
                })
                .p(px(20.0))
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    image(move || app.art.get())
                        .size(move || px((app.lyric_panel_height.get() * 0.4).max(120.0)))
                        .rounded(px(9.0))
                        .shadow_lg(),
                )
                .child(dynamic(move || {
                    let label = format!(
                        "{} {}",
                        localized(&app.language.get(), "Artist:"),
                        app.current.get().map_or_else(
                            || localized(&app.language.get(), "Unknown"),
                            |song| song.singer
                        )
                    );
                    let label = text(label)
                        .mt(px(6.0))
                        .w(px((app.lyric_panel_height.get() * 0.4).max(120.0)))
                        .text_size(px(14.0))
                        .text_color(move || theme().foreground);
                    if app.lyrics.get().is_empty() {
                        label.text_center()
                    } else {
                        label.text_left()
                    }
                }))
                .child(dynamic(move || {
                    let label = format!(
                        "{} {}",
                        localized(&app.language.get(), "Title:"),
                        app.current.get().map_or_else(
                            || localized(&app.language.get(), "No song"),
                            |song| song.song_name
                        )
                    );
                    let label = text(label)
                        .w(px((app.lyric_panel_height.get() * 0.4).max(120.0)))
                        .text_size(px(14.0))
                        .text_color(move || theme().foreground);
                    if app.lyrics.get().is_empty() {
                        label.text_center()
                    } else {
                        label.text_left()
                    }
                })),
        )
        .child(
            when(
                move || !app.lyrics.get().is_empty(),
                move || {
                    scroll()
                        .test_id("lyrics-list")
                        .w(percentage(0.7))
                        .flex_1()
                        .min_w(px(0.0))
                        .element_ref(&app.lyric_scroll)
                        .on_scroll(move |_event: &ScrollEvent| {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;
                            if now >= app.ignore_programmatic_scroll_until.get() {
                                app.manual_lyrics_until.set(now + 5000);
                                app.last_lyric_index.set(-1);
                            }
                        })
                        .child(view().pt(px(4.0)).pb(px(12.0)).flex_col().child(each(
                            move || app.lyrics.get(),
                            |line: &LyricItem| format!("{}:{}", line.time, line.text),
                            move |line: ReadSignal<LyricItem>, _index: ReadSignal<usize>| {
                                let line = line.get();
                                let line_start = line.time;
                                let line_end = line.time + line.duration;
                                let line_text = line.text.clone();
                                let active = computed(move || {
                                    app.position.get() >= line_start
                                        && app.position.get() < line_end
                                });
                                view()
                                    .h(px(40.0))
                                    .items_center()
                                    .justify_center()
                                    .on_press(move || {
                                        app.position.set(line.time);
                                        Music::seek(line.time);
                                    })
                                    .child(dynamic(move || {
                                        if active.get() {
                                            text(line_text.clone())
                                                .text_size(px(20.0))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme().primary)
                                        } else {
                                            text(line_text.clone())
                                                .text_size(px(17.0))
                                                .font_weight(FontWeight::NORMAL)
                                                .text_color(theme().foreground)
                                        }
                                    }))
                            },
                        )))
                },
            )
            .otherwise(|| view()),
        )
}

fn settings_page(app: UiState, refresh: impl Fn(String) + Copy + 'static) -> impl IntoElement {
    let toggle_spectrum = move || {
        let next = !app.show_spectrum.get();
        app.show_spectrum.set(next);
        Music::set_spectrum_enabled(next);
        let _ = Storage::set_item("zeedle.show-spectrum", next.to_string());
    };
    view()
        .test_id("screen-3")
        .flex_1()
        .min_h(px(0.0))
        .bg(move || color_token("page"))
        .child(
            scroll()
                .flex_1()
                .p(px(24.0))
                .gap(px(20.0))
                .child(
                    view()
                        .p(px(16.0))
                        .flex_col()
                        .gap(px(10.0))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(move || color_token("border"))
                        .bg(move || color_token("panel"))
                        .child(
                            text(move || localized(&app.language.get(), "Playback"))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || color_token("text-tertiary")),
                        )
                        .child(view().h(px(1.0)).bg(move || color_token("divider")))
                        .child(
                            view()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    text(move || localized(&app.language.get(), "Music directory"))
                                        .text_size(px(13.0))
                                        .text_color(move || color_token("text-secondary")),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            view()
                                                .flex_1()
                                                .min_w(px(0.0))
                                                .h(px(32.0))
                                                .justify_center()
                                                .px(px(12.0))
                                                .rounded(px(4.0))
                                                .border_1()
                                                .border_color(move || color_token("input-border"))
                                                .bg(move || color_token("raised"))
                                                .child(
                                                    text(move || app.directory.get())
                                                        .text_size(px(13.0))
                                                        .text_color(move || {
                                                            color_token("text-secondary")
                                                        }),
                                                ),
                                        )
                                        .child(
                                            view()
                                                .px(px(12.0))
                                                .h(px(32.0))
                                                .items_center()
                                                .justify_center()
                                                .rounded(px(4.0))
                                                .bg(move || color_token("button"))
                                                .on_press(move || {
                                                    spawn(async move {
                                                        let path = Music::choose_directory().await;
                                                        if !path.is_empty() {
                                                            refresh(path);
                                                        }
                                                    });
                                                })
                                                .child(
                                                    text(move || {
                                                        localized(&app.language.get(), "Browse…")
                                                    })
                                                    .text_size(px(13.0))
                                                    .text_color(move || theme().foreground),
                                                ),
                                        ),
                                ),
                        )
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(10.0))
                                .child(checkbox(app.show_spectrum, toggle_spectrum))
                                .child(
                                    text(move || {
                                        localized(&app.language.get(), "Spectrum display")
                                    })
                                    .on_press(toggle_spectrum)
                                    .text_size(px(13.0))
                                    .text_color(move || theme().foreground),
                                ),
                        ),
                )
                .child(
                    view()
                        .p(px(16.0))
                        .flex_col()
                        .gap(px(10.0))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(move || color_token("border"))
                        .bg(move || color_token("panel"))
                        .child(
                            text(move || localized(&app.language.get(), "Appearance"))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || color_token("text-tertiary")),
                        )
                        .child(view().h(px(1.0)).bg(move || color_token("divider")))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(10.0))
                                .child(checkbox(app.follow_system, move || {
                                    let next = !app.follow_system.get();
                                    app.follow_system.set(next);
                                    if next {
                                        nsh_core::element::ambient::set_color_scheme(
                                            Appearance::System,
                                        );
                                        let _ = Storage::remove_item("zeedle.light-theme");
                                    } else {
                                        let current_light = !nsh_core::element::ambient::dark();
                                        app.light.set(current_light);
                                        let _ = Storage::set_item(
                                            "zeedle.light-theme",
                                            app.light.get().to_string(),
                                        );
                                        nsh_core::element::ambient::set_color_scheme(
                                            if app.light.get() {
                                                Appearance::Light
                                            } else {
                                                Appearance::Dark
                                            },
                                        );
                                    }
                                }))
                                .child(
                                    text(move || {
                                        localized(&app.language.get(), "Follow system theme")
                                    })
                                    .on_press(move || {
                                        let next = !app.follow_system.get();
                                        app.follow_system.set(next);
                                        if next {
                                            nsh_core::element::ambient::set_color_scheme(
                                                Appearance::System,
                                            );
                                            let _ = Storage::remove_item("zeedle.light-theme");
                                        } else {
                                            let current_light = !nsh_core::element::ambient::dark();
                                            app.light.set(current_light);
                                            let _ = Storage::set_item(
                                                "zeedle.light-theme",
                                                app.light.get().to_string(),
                                            );
                                            nsh_core::element::ambient::set_color_scheme(
                                                if app.light.get() {
                                                    Appearance::Light
                                                } else {
                                                    Appearance::Dark
                                                },
                                            );
                                        }
                                    })
                                    .text_size(px(13.0))
                                    .text_color(move || theme().foreground),
                                ),
                        )
                        .child(
                            text(move || localized(&app.language.get(), "Language"))
                                .text_size(px(13.0))
                                .text_color(move || color_token("text-secondary")),
                        )
                        .child(
                            view()
                                .flex_row()
                                .gap(px(8.0))
                                .child(language_button("English", "en", app.language))
                                .child(language_button("中文", "zh", app.language))
                                .child(language_button("Español", "es", app.language))
                                .child(language_button("Français", "fr", app.language))
                                .child(language_button("Deutsch", "de", app.language))
                                .child(language_button("Русский", "ru", app.language)),
                        ),
                ),
        )
}

fn checkbox(checked: Signal<bool>, on: impl Fn() + 'static) -> impl IntoElement {
    view()
        .w(px(18.0))
        .h(px(18.0))
        .rounded(px(2.0))
        .border_1()
        .border_color(move || {
            if checked.get() {
                theme().primary
            } else {
                color_token("checkbox-border")
            }
        })
        .bg(move || {
            if checked.get() {
                theme().primary
            } else {
                color_token("page")
            }
        })
        .items_center()
        .justify_center()
        .on_press(on)
        .child(when(checked, || svg_icon("check", 15.0, "primary-foreground")).otherwise(|| view()))
}

fn language_button(
    label: &'static str,
    code: &'static str,
    language: Signal<String>,
) -> impl IntoElement {
    view()
        .flex_1()
        .min_w(px(0.0))
        .h(px(26.0))
        .items_center()
        .justify_center()
        .rounded(px(4.0))
        .border_1()
        .border_color(move || {
            if language.get() == code {
                theme().primary
            } else {
                color_token("input-border")
            }
        })
        .bg(move || {
            if language.get() == code {
                theme().primary
            } else {
                color_token("hover")
            }
        })
        .on_press(move || {
            language.set(code.to_string());
            let _ = Storage::set_item("zeedle.language", code);
        })
        .child(text(label).text_size(px(12.0)).text_color(move || {
            if language.get() == code {
                theme().primary_foreground
            } else {
                theme().foreground
            }
        }))
}

fn about_page(app: UiState) -> impl IntoElement {
    view()
        .test_id("screen-4")
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .bg(move || color_token("page"))
        .child(image(COVER.uri).size(px(120.0)))
        .child(
            text(move || {
                format!(
                    "Zeedle\n{}",
                    localized(
                        &app.language.get(),
                        "Music at the speed of a blink, designed for pure simplicity"
                    )
                )
            })
            .text_size(px(14.0))
            .text_color(move || theme().foreground)
            .text_center(),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(4.0))
                .child(
                    text(move || localized(&app.language.get(), "Built by"))
                        .text_size(px(14.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .on_press(move || {
                            let error =
                                Music::open_url("https://github.com/RustAudio/rodio".to_string());
                            if !error.is_empty() {
                                app.error.set(error);
                            }
                        })
                        .child(
                            text("Rodio")
                                .text_size(px(14.0))
                                .text_color(move || theme().primary)
                                .underline(),
                        ),
                )
                .child(
                    text(move || localized(&app.language.get(), "and"))
                        .text_size(px(14.0))
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .test_id("about-nsh-link")
                        .on_press(move || {
                            let error = Music::open_url("https://nsh.dev".to_string());
                            if !error.is_empty() {
                                app.error.set(error);
                            }
                        })
                        .child(
                            text("NSH")
                                .text_size(px(14.0))
                                .text_color(move || theme().primary)
                                .underline(),
                        ),
                ),
        )
        .child(
            text(move || {
                format!(
                    "{} Jordan Haidee\n{} 0.1.1",
                    localized(&app.language.get(), "Author:"),
                    localized(&app.language.get(), "Version:")
                )
            })
            .text_size(px(14.0))
            .text_color(move || theme().foreground)
            .text_center(),
        )
        .child(
            view()
                .on_press(move || {
                    let error =
                        Music::open_url("https://github.com/Jordan-Haidee/Zeedle".to_string());
                    if !error.is_empty() {
                        app.error.set(error);
                    }
                })
                .child(
                    text(move || localized(&app.language.get(), "Github"))
                        .text_size(px(14.0))
                        .text_color(move || theme().primary)
                        .underline(),
                ),
        )
}

fn player_panel(app: UiState) -> impl IntoElement {
    let volume_hovered = signal(false);
    let set_volume = move |value: f32| {
        let value = value.clamp(0.0, 1.0);
        app.volume.set(value);
        Music::set_volume(value);
        let _ = Storage::set_item("zeedle.volume", value.to_string());
    };
    let set_volume_at = move |y: f32| set_volume(1.0 - y / 55.0);
    let progress = move || {
        if app.duration.get() <= 0.0 {
            0.0
        } else {
            (app.position.get() / app.duration.get()).clamp(0.0, 1.0)
        }
    };
    let toggle = move || {
        spawn(async move {
            let next = !app.paused.get();
            app.paused.set(Music::set_paused(next).await);
        });
    };
    view()
        .h(px(72.0))
        .flex_col()
        .bg(move || color_token("page"))
        .child(
            view()
                .test_id("progress-seek")
                .h(px(4.0))
                .mx(px(10.0))
                .bg(move || color_token("track"))
                .on_layout(move |bounds: &Bounds<Points>| app.track_width.set(bounds.size.width.0))
                .on(EventKind::Press, move |payload: &EventPayload| {
                    if let EventPayload::Press(press) = payload {
                        let target = (press.position.x.0 / app.track_width.get()).clamp(0.0, 1.0)
                            * app.duration.get();
                        app.position.set(target);
                        Music::seek(target);
                    }
                })
                .child(
                    view()
                        .h(px(3.0))
                        .w(move || px(progress() * app.track_width.get()))
                        .bg(move || theme().primary),
                )
                .child(
                    view()
                        .absolute()
                        .left(move || px((progress() * app.track_width.get() - 10.0).max(0.0)))
                        .top(px(-8.0))
                        .size(px(20.0))
                        .rounded(px(10.0))
                        .border(px(3.0))
                        .border_color(move || color_token("track-thumb-border"))
                        .bg(move || theme().primary),
                ),
        )
        .child(
            view()
                .h(px(68.0))
                .flex_row()
                .items_center()
                .child(
                    view()
                        .w(percentage(0.35))
                        .flex_row()
                        .items_center()
                        .gap(px(3.0))
                        .on_press(move || app.page.set(2))
                        .child(
                            view()
                                .w(percentage(0.2))
                                .items_center()
                                .justify_center()
                                .child(
                                    image(move || app.art.get()).size(px(40.0)).rounded(px(4.0)),
                                ),
                        )
                        .child(
                            text(move || {
                                app.current.get().map_or_else(
                                    || localized(&app.language.get(), "No song selected"),
                                    |song| format!("{} - {}", song.song_name, song.singer),
                                )
                            })
                            .w(percentage(0.8))
                            .text_size(px(14.0))
                            .text_color(move || theme().foreground)
                            .text_center(),
                        ),
                )
                .child(
                    view()
                        .w(percentage(0.06))
                        .h(px(68.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .on_press(move || {
                            let next = if app.mode.get() == "order" {
                                "repeat"
                            } else {
                                "order"
                            };
                            app.mode.set(next.to_string());
                            let _ = Storage::set_item("zeedle.play-mode", next);
                        })
                        .child(
                            when(
                                move || app.mode.get() != "random",
                                move || {
                                    when(
                                        move || app.mode.get() == "repeat",
                                        || svg_icon("repeat-2", 18.0, "primary"),
                                    )
                                    .otherwise(|| svg_icon("repeat", 18.0, "primary"))
                                },
                            )
                            .otherwise(|| svg_icon("repeat", 18.0, "text-secondary")),
                        ),
                )
                .child(
                    view()
                        .w(percentage(0.06))
                        .h(px(68.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .child(
                            view()
                                .test_id("control-previous")
                                .size(px(36.0))
                                .min_w(px(36.0))
                                .min_h(px(36.0))
                                .relative()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .hover(|style| style.bg(move || color_token("hover")))
                                .active(|style| style.bg(move || color_token("pressed")))
                                .on_press(move || play_previous(app))
                                .child(transport_icon("skip-back", "text-secondary")),
                        ),
                )
                .child(
                    view()
                        .w(percentage(0.06))
                        .h(px(68.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .child(
                            view()
                                .test_id("control-play-pause")
                                .size(px(36.0))
                                .min_w(px(36.0))
                                .min_h(px(36.0))
                                .relative()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .rounded(px(9.0))
                                .hover(|style| style.bg(move || color_token("hover")))
                                .active(|style| style.bg(move || color_token("pressed")))
                                .on_press(toggle)
                                .child(
                                    when(
                                        move || app.paused.get(),
                                        || transport_icon("play", "primary"),
                                    )
                                    .otherwise(|| transport_icon("pause", "primary")),
                                ),
                        ),
                )
                .child(
                    view()
                        .w(percentage(0.06))
                        .h(px(68.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .child(
                            view()
                                .test_id("control-next")
                                .size(px(36.0))
                                .min_w(px(36.0))
                                .min_h(px(36.0))
                                .relative()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .hover(|style| style.bg(move || color_token("hover")))
                                .active(|style| style.bg(move || color_token("pressed")))
                                .on_press(move || play_next(app))
                                .child(transport_icon("skip-forward", "text-secondary")),
                        ),
                )
                .child(
                    view()
                        .w(percentage(0.06))
                        .h(px(68.0))
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .on_press(move || {
                            let next = if app.mode.get() == "random" {
                                "order"
                            } else {
                                "random"
                            };
                            app.mode.set(next.to_string());
                            let _ = Storage::set_item("zeedle.play-mode", next);
                        })
                        .child(
                            when(
                                move || app.mode.get() == "random",
                                || svg_icon("shuffle", 18.0, "primary"),
                            )
                            .otherwise(|| svg_icon("shuffle", 18.0, "text-secondary")),
                        ),
                )
                .child(
                    text(move || {
                        format!(
                            "{} / {}",
                            time_label(app.position.get()),
                            time_label(app.duration.get())
                        )
                    })
                    .w(percentage(0.25))
                    .text_size(px(14.0))
                    .text_color(move || theme().foreground)
                    .text_center(),
                )
                .child(
                    view()
                        .w(percentage(0.1))
                        .h(px(68.0))
                        .relative()
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .on_hover_in(move || volume_hovered.set(true))
                        .on_hover_out(move || volume_hovered.set(false))
                        .child(
                            when(volume_hovered, move || {
                                view()
                                    .absolute()
                                    .right(px(13.0))
                                    .top(px(6.0))
                                    .w(px(20.0))
                                    .h(px(55.0))
                                    .on(EventKind::Press, move |payload: &EventPayload| {
                                        if let EventPayload::Press(press) = payload {
                                            set_volume_at(press.position.y.0);
                                        }
                                    })
                                    .child(
                                        view()
                                            .pointer_events(PointerEvents::None)
                                            .absolute()
                                            .left(px(8.0))
                                            .top(px(0.0))
                                            .w(px(4.0))
                                            .h(px(55.0))
                                            .rounded(px(2.0))
                                            .bg(move || color_token("track"))
                                            .child(
                                                view()
                                                    .absolute()
                                                    .left(px(0.0))
                                                    .bottom(px(0.0))
                                                    .w(px(4.0))
                                                    .h(move || px(55.0 * app.volume.get()))
                                                    .rounded(px(2.0))
                                                    .bg(move || theme().primary),
                                            )
                                            .child(
                                                view()
                                                    .absolute()
                                                    .left(px(-3.0))
                                                    .bottom(move || {
                                                        px((55.0 * app.volume.get() - 5.0)
                                                            .clamp(0.0, 50.0))
                                                    })
                                                    .size(px(10.0))
                                                    .rounded(px(5.0))
                                                    .border_1()
                                                    .border_color(move || color_token("page"))
                                                    .bg(move || theme().primary),
                                            ),
                                    )
                            })
                            .otherwise(|| view()),
                        )
                        .child(
                            view()
                                .w(px(28.0))
                                .h(px(30.0))
                                .items_center()
                                .justify_center()
                                .hover(|style| style.bg(move || color_token("hover")))
                                .active(|style| style.bg(move || color_token("pressed")))
                                .on_press(move || {
                                    let next = if app.volume.get() > 0.0 { 0.0 } else { 0.8 };
                                    set_volume(next);
                                    let _ = Storage::set_item("zeedle.volume", next.to_string());
                                })
                                .child(
                                    when(
                                        move || volume_hovered.get(),
                                        || svg_icon("volume-2", 20.0, "primary"),
                                    )
                                    .otherwise(|| svg_icon("volume-2", 20.0, "text-secondary")),
                                ),
                        ),
                ),
        )
}
