import {
  cleanup,
  effect,
  Image,
  LayoutAnimation,
  List,
  Scroll,
  Storage,
  View,
  colorScheme,
  keymap,
  setColorScheme,
  signal,
  token,
  type InputRef,
  type ListRef,
  type ViewRef,
} from "nsh";
import { Checkbox as NshCheckbox, Input, TitleBar } from "nsh/base";
import { ProgressSlider } from "./components/ProgressSlider";
import { PlayerTransportButton, VolumeControl } from "./components/PlayerControls";
import { Text } from "./components/Text";
import { Application, currentWindow, Menu, Tray, Window } from "nsh/desktop";
import { AppConfig, Music } from "zeedle-nsh/native";
import { LucideIcon, type LucideName } from "./icons";
import cover from "../assets/cover.svg";
import { languageName, parseLanguage, translate, type Language } from "./translations";
import { createTrayOptions } from "./tray";
import type {
  AppConfigValue,
  LyricItem,
  Page,
  PlaybackState,
  PlayMode,
  SongInfo,
  SortField,
} from "./types";
import { sameSongPath, timeLabel } from "./format";
import { selectNextSong, selectPreviousSong } from "./playback";
import { closeWindow } from "./window";

const contentCornerStyle = {
  borderRadius: 10,
  borderTopRightRadius: 0,
  borderBottomRightRadius: 0,
  borderBottomLeftRadius: 0,
};

function AppWindow() {
  const startupStartedAt = Date.now();
  const config = JSON.parse(AppConfig.load()) as AppConfigValue;
  const appFontFamily = AppConfig.defaultFontFamily();
  const win = currentWindow();
  const page = signal<Page>(Math.max(0, Math.min(4, config.current_page)) as Page);
  const expanded = signal(config.sidebar_expanded);
  const closeToTray = signal(config.close_to_tray);
  const light = () => colorScheme() === "light";
  const followSystem = signal(config.follow_system_theme);
  const language = signal<Language>(parseLanguage(config.lang));
  const directory = signal(config.song_dir || Music.defaultDirectory());
  const songs = signal<SongInfo[]>([]);
  const current = signal<SongInfo | null>(null);
  const lyrics = signal<LyricItem[]>([]);
  const art = signal<string | typeof cover>(cover);
  const query = signal("");
  const search = signal("");
  const searchResults = signal<SongInfo[]>([]);
  const loading = signal(false);
  const sortField = signal<SortField>(
    config.sort_key === "BySinger"
      ? "singer"
      : config.sort_key === "ByDuration"
        ? "duration_secs"
        : "song_name",
  );
  const ascending = signal(config.sort_ascending);
  const mode = signal<PlayMode>(
    config.play_mode === "Recursive"
      ? "repeat"
      : config.play_mode === "Random"
        ? "random"
        : "order",
  );
  const position = signal(config.progress);
  const duration = signal(0);
  const paused = signal(true);
  const playerLoaded = signal(false);
  const volume = signal(config.volume);
  const showSpectrum = signal(config.show_spectrum);
  const galleryViewportY = signal(config.gallery_viewport_y);
  const requestedSongPath = config.current_song_path;
  const spectrumBars = signal<number[]>(Array.from({ length: 48 }, () => 0));
  const playerError = signal("");
  const seeking = signal(false);
  let pendingSeekPosition: number | null = null;
  let pendingSeekUntil = 0;
  const trackWidth = signal(1);
  const history = signal<string[]>([]);
  const historyIndex = signal(-1);
  let searchInputRef: InputRef | undefined;
  let contentPaneRef: ViewRef | undefined;
  const editingSong = signal<SongInfo | null>(null);
  const editTitle = signal("");
  const editArtist = signal("");
  const editAlbum = signal("");
  const editCoverPath = signal("");
  const editCoverName = signal("");
  const editCoverPreview = signal<string | typeof cover>(cover);
  const editLyricsPath = signal("");
  const editLyricsName = signal("");
  const editLyricsPreview = signal("");
  const editError = signal("");
  const editSaving = signal(false);
  const deleteTarget = signal<SongInfo | null>(null);
  let lyricListRef!: ListRef;
  let lastAutoScrolledLyric = -1;
  let manualLyricsScrollUntil = 0;
  let ignoreProgrammaticLyricsScrollUntil = 0;
  let lastLyricsScrollOffset: number | null = null;
  let handledEnd = false;

  const t = (key: string) => translate(language(), key);
  function handleWindowClose() {
    saveConfig();
    closeWindow(
      closeToTray(),
      () => win.hide(),
      () => Application.quit(),
    );
  }
  win.on("closePress", handleWindowClose);
  async function openExternalUrl(url: string) {
    const error = await Music.openUrl(url);
    if (error) playerError.set(error);
  }

  if (config.follow_system_theme) setColorScheme("system");
  else setColorScheme(config.light_ui ? "light" : "dark");

  const displayedSongs = () => {
    const items = [...songs()];
    const field = sortField();
    const dir = ascending() ? 1 : -1;
    return items.sort((a, b) => {
      const left = a[field];
      const right = b[field];
      return (
        (typeof left === "number" && typeof right === "number"
          ? left - right
          : String(left).localeCompare(String(right), undefined, { sensitivity: "base" })) * dir
      );
    });
  };

  const searchSongs = (rawQuery = search()) => {
    const term = rawQuery.trim().toLocaleLowerCase();
    if (!term) return [];
    return songs()
      .map((song, index) => {
        const title = song.song_name.toLocaleLowerCase();
        const artist = song.singer.toLocaleLowerCase();
        const score =
          (title === term ? 100 : title.startsWith(term) ? 80 : title.includes(term) ? 60 : 0) +
          (artist === term ? 50 : artist.startsWith(term) ? 40 : artist.includes(term) ? 20 : 0);
        return { song, score, index };
      })
      .filter((entry) => entry.score > 0)
      .sort((a, b) => b.score - a.score || a.index - b.index)
      .map((entry) => entry.song);
  };

  function updateSearch(value: string) {
    query.set(value);
    search.set(value);
    searchResults.set(searchSongs(value));
  }

  async function refreshSongs(path = directory()) {
    loading.set(true);
    playerError.set("");
    try {
      const result = JSON.parse(await Music.scan(path)) as SongInfo[];
      songs.set(result);
      searchResults.set(searchSongs(search()));
      directory.set(path);
      const updatedCurrent = result.find((song) => song.song_path === current()?.song_path);
      if (updatedCurrent) current.set(updatedCurrent);
      if (!result.length) {
        Music.stop();
        playerLoaded.set(false);
        current.set(null);
        lyrics.set([]);
        art.set(cover);
        position.set(0);
        duration.set(0);
        return;
      }
      const active =
        result.find((song) =>
          sameSongPath(song.song_path, current()?.song_path ?? requestedSongPath),
        ) ?? result[0];
      if (!current() || !result.some((song) => song.song_path === current()?.song_path)) {
        Music.stop();
        playerLoaded.set(true);
        current.set(active);
        const restoreProgress = sameSongPath(active.song_path, requestedSongPath)
          ? Math.max(0, Math.min(config.progress, active.duration_secs))
          : 0;
        position.set(restoreProgress);
        duration.set(active.duration_secs);
        paused.set(true);
        lastAutoScrolledLyric = -1;
        Music.load(active.song_path);
        if (restoreProgress > 0) Music.seek(restoreProgress);
        const [lyricsJson, nextArt] = await Promise.all([
          Music.lyrics(active.song_path),
          Music.cover(active.song_path),
        ]);
        if (current()?.song_path === active.song_path) {
          lyrics.set(JSON.parse(lyricsJson) as LyricItem[]);
          art.set(nextArt || cover);
        }
      }
    } catch (error) {
      playerError.set(String(error));
    } finally {
      loading.set(false);
    }
  }

  Tray.set(createTrayOptions(win, () => void refreshSongs(), t));

  async function playSong(song: SongInfo, addHistory = true) {
    current.set(song);
    lastAutoScrolledLyric = -1;
    playerLoaded.set(true);
    position.set(0);
    duration.set(song.duration_secs);
    paused.set(false);
    playerError.set("");
    handledEnd = false;
    if (addHistory) {
      const next = history().slice(0, historyIndex() + 1);
      next.push(song.song_path);
      history.set(next);
      historyIndex.set(next.length - 1);
    }
    Music.play(song.song_path);
    const [lyricsJson, nextArt] = await Promise.all([
      Music.lyrics(song.song_path),
      Music.cover(song.song_path),
    ]);
    const nextLyrics = JSON.parse(lyricsJson) as LyricItem[];
    if (current()?.song_path === song.song_path) {
      lyrics.set(nextLyrics);
      art.set(nextArt || cover);
    }
  }

  function playNext() {
    const list = songs();
    const nextHistoryIndex = historyIndex() + 1;
    const nextHistorySong = list.find((song) => song.song_path === history()[nextHistoryIndex]);
    const song = selectNextSong(
      list,
      current()?.song_path ?? null,
      mode(),
      history(),
      historyIndex(),
    );
    if (!song) return;
    if (mode() === "random" && nextHistorySong?.song_path === song.song_path) {
      historyIndex.set(nextHistoryIndex);
      void playSong(song, false);
      return;
    }
    void playSong(song, mode() === "repeat" ? false : true);
  }

  function playPrevious() {
    const list = songs();
    const previousHistoryIndex = historyIndex() - 1;
    const previousHistorySong = list.find(
      (song) => song.song_path === history()[previousHistoryIndex],
    );
    if (mode() === "random" && previousHistorySong) {
      historyIndex.set(previousHistoryIndex);
      void playSong(previousHistorySong, false);
      return;
    }
    const song = selectPreviousSong(
      list,
      current()?.song_path ?? null,
      mode(),
      history(),
      historyIndex(),
    );
    if (song) void playSong(song);
  }

  const pausePending = signal(false);
  async function togglePlay() {
    if (pausePending()) return;
    if (!current()) {
      const first = displayedSongs()[0];
      if (first) void playSong(first);
      return;
    }
    if (!playerLoaded()) {
      void playSong(current()!, false);
      return;
    }
    const target = !paused();
    pausePending.set(true);
    try {
      paused.set(await Music.setPaused(target));
    } finally {
      pausePending.set(false);
    }
  }

  function changeMode(next: "order" | "repeat" | "random") {
    mode.set(next);
  }

  function seekTo(seconds: number) {
    const next = Math.max(0, Math.min(duration(), seconds));
    pendingSeekPosition = next;
    pendingSeekUntil = Date.now() + 1500;
    position.set(next);
    Music.seek(next);
  }

  function progressAtX(x: number) {
    const width = trackWidth();
    return width > 0 && duration() > 0 ? Math.max(0, Math.min(1, x / width)) * duration() : 0;
  }

  function noteManualLyricsScroll() {
    manualLyricsScrollUntil = Date.now() + 5000;
    lastAutoScrolledLyric = -1;
  }

  function observeLyricsScroll(offset: number) {
    const changed =
      lastLyricsScrollOffset !== null && Math.abs(offset - lastLyricsScrollOffset) > 2;
    lastLyricsScrollOffset = offset;
    if (changed && Date.now() >= ignoreProgrammaticLyricsScrollUntil) noteManualLyricsScroll();
  }

  function autoScrollLyrics(seconds: number) {
    if (page() !== 2 || paused() || Date.now() < manualLyricsScrollUntil || !lyricListRef) return;
    const activeIndex = lyricIndexAt(seconds);
    if (activeIndex < 0 || activeIndex === lastAutoScrolledLyric) return;
    lastAutoScrolledLyric = activeIndex;
    ignoreProgrammaticLyricsScrollUntil = Date.now() + 350;
    lyricListRef.scrollToIndex({ index: Math.max(0, activeIndex - 5), animated: false });
  }

  function lyricIndexAt(seconds: number) {
    let activeIndex = -1;
    for (let index = 0; index < lyrics().length; index++) {
      if (lyrics()[index].time > seconds) break;
      activeIndex = index;
    }
    return activeIndex;
  }

  function seekBy(seconds: number) {
    if (page() === 2 && lyrics().length) {
      const target =
        seconds < 0
          ? ([...lyrics()].reverse().find((line) => line.time < position() - 0.5)?.time ?? 0)
          : (lyrics().find((line) => line.time > position() + 0.5)?.time ?? position());
      seekTo(target);
    } else {
      seekTo(position() + seconds);
    }
  }

  function navigateToPage(target: Page) {
    searchInputRef?.blur();
    if (target === 2) {
      lastAutoScrolledLyric = -1;
      lastLyricsScrollOffset = null;
      ignoreProgrammaticLyricsScrollUntil = Date.now() + 500;
    }
    page.set(target);
    contentPaneRef?.focus();
  }

  const shortcuts = {
    space: togglePlay,
    left: () => seekBy(-15),
    right: () => seekBy(15),
    up: playPrevious,
    down: playNext,
    f1: () => navigateToPage(0),
    f2: () => navigateToPage(1),
    f3: () => navigateToPage(2),
    f4: () => navigateToPage(3),
    f5: () => navigateToPage(4),
  };

  keymap.bind(shortcuts, { when: "notEditing", scope: "window" });

  async function chooseDirectory() {
    const path = await Music.chooseDirectory();
    if (path) await refreshSongs(path);
  }

  async function beginEdit(song: SongInfo) {
    editingSong.set(song);
    editTitle.set(song.song_name);
    editArtist.set(song.singer);
    editAlbum.set("");
    editCoverPath.set("");
    editCoverName.set("");
    editCoverPreview.set(cover);
    editLyricsPath.set("");
    editLyricsName.set("");
    editLyricsPreview.set("");
    editError.set("");
    try {
      const metadata = JSON.parse(await Music.editorMetadata(song.song_path)) as {
        title?: string;
        artist?: string;
        album?: string;
        cover?: string;
        lyrics?: string;
      };
      if (editingSong()?.song_path === song.song_path) {
        editTitle.set(metadata.title || song.song_name);
        editArtist.set(metadata.artist || song.singer);
        editAlbum.set(metadata.album || "");
        editCoverPreview.set(metadata.cover || cover);
        editCoverName.set(metadata.cover ? t("Embedded artwork") : t("No cover image"));
        editLyricsPreview.set(metadata.lyrics || "");
        editLyricsName.set(metadata.lyrics ? t("Embedded lyrics") : t("No embedded lyrics"));
      }
    } catch (error) {
      editError.set(String(error));
    }
  }

  async function chooseEditCover() {
    const path = await Music.chooseCover();
    if (!path) return;
    const preview = await Music.coverFile(path);
    if (!preview) {
      editError.set(t("Could not read that image file."));
      return;
    }
    editCoverPath.set(path);
    editCoverName.set(path.split(/[\\/]/).pop() || path);
    editCoverPreview.set(preview);
    editError.set("");
  }

  async function chooseEditLyrics() {
    const path = await Music.chooseLyrics();
    if (!path) return;
    const text = await Music.lyricsFile(path);
    if (!text) {
      editError.set(t("Could not read that LRC file."));
      return;
    }
    editLyricsPath.set(path);
    editLyricsName.set(path.split(/[\\/]/).pop() || path);
    editLyricsPreview.set(text);
    editError.set("");
  }

  async function saveEdit() {
    const song = editingSong();
    if (!song || editSaving()) return;
    if (!editTitle().trim() || !editArtist().trim()) {
      editError.set(t("Title and artist are required."));
      return;
    }
    editSaving.set(true);
    editError.set("");
    try {
      const error = await Music.updateMetadata(
        song.song_path,
        editTitle(),
        editArtist(),
        editAlbum(),
        editCoverPath(),
        editLyricsPath(),
      );
      if (error) {
        editError.set(error);
        return;
      }
      editingSong.set(null);
      await refreshSongs();
    } catch (error) {
      editError.set(String(error));
    } finally {
      editSaving.set(false);
    }
  }

  async function confirmDelete() {
    const song = deleteTarget();
    if (!song) return;
    const error = await Music.deleteSong(song.song_path);
    deleteTarget.set(null);
    if (error) {
      playerError.set(error);
      return;
    }
    await refreshSongs();
  }

  void (async () => {
    await Promise.all(
      [
        "zeedle.music-directory",
        "zeedle.volume",
        "zeedle.play-mode",
        "zeedle.show-spectrum",
        "zeedle.language",
        "zeedle.light-theme",
      ].map((key) => Storage.removeItem(key)),
    );
    Music.setVolume(volume());
    Music.setSpectrumEnabled(showSpectrum());
    await refreshSongs(directory());
  })()
    .catch((error) => playerError.set(String(error)))
    .finally(() => {
      win.show();
      Music.logFirstScreenReady(Math.max(0, Date.now() - startupStartedAt));
    });

  function saveConfig() {
    const snapshot: AppConfigValue = {
      ...config,
      song_dir: directory(),
      current_song_path: current()?.song_path ?? null,
      progress: position(),
      play_mode: mode() === "repeat" ? "Recursive" : mode() === "random" ? "Random" : "InOrder",
      sort_key:
        sortField() === "singer"
          ? "BySinger"
          : sortField() === "duration_secs"
            ? "ByDuration"
            : "BySongName",
      sort_ascending: ascending(),
      lang: language() === "en" ? "" : language(),
      light_ui: light(),
      follow_system_theme: followSystem(),
      volume: volume(),
      show_spectrum: showSpectrum(),
      current_page: page(),
      sidebar_expanded: expanded(),
      gallery_viewport_y: galleryViewportY(),
      close_to_tray: closeToTray(),
    };
    const error = AppConfig.save(JSON.stringify(snapshot));
    if (error) console.error(`failed to save config: ${error}`);
  }

  let firstConfigEffectRun = true;
  effect(() => {
    page();
    expanded();
    directory();
    current()?.song_path;
    mode();
    sortField();
    ascending();
    language();
    light();
    followSystem();
    volume();
    showSpectrum();
    galleryViewportY();
    closeToTray();
    if (firstConfigEffectRun) {
      firstConfigEffectRun = false;
      return;
    }
    const timer = setTimeout(saveConfig, 250);
    return () => clearTimeout(timer);
  });
  cleanup(saveConfig);

  setInterval(() => {
    const state = JSON.parse(Music.state()) as PlaybackState;
    if (!state.paused && state.duration > 0) {
      if (pendingSeekPosition !== null) {
        if (
          Date.now() >= pendingSeekUntil ||
          Math.abs(state.position - pendingSeekPosition) <= 0.5
        ) {
          pendingSeekPosition = null;
        }
      }
      if (!seeking() && pendingSeekPosition === null) position.set(state.position);
      duration.set(state.duration);
    }
    if (!pausePending()) paused.set(state.paused);
    autoScrollLyrics(state.position);
    if (state.error) playerError.set(state.error);
    volume.set(state.volume);
    spectrumBars.set(state.spectrum);
    if (
      !state.paused &&
      state.duration > 0 &&
      state.position >= state.duration - 0.12 &&
      !handledEnd
    ) {
      handledEnd = true;
      playNext();
    }
  }, 120);

  const textColor = () => token("foreground");
  const secondaryColor = () => token("text-secondary");
  const mutedColor = () => token("text-tertiary");
  const accentColor = () => token("primary");
  const dangerColor = () => token("danger");
  const dangerSurfaceColor = () => token("danger-surface");
  const dangerTextColor = () => token("danger-text");
  const shadowColor = () => token("shadow");

  const textStyle = (
    size = 14,
    color = textColor(),
    weight: "400" | "500" | "600" | "700" = "400",
  ) => ({
    fontSize: size,
    color,
    fontWeight: weight,
    fontFamily: appFontFamily,
  });

  const controlStyle = (size = 36, radius = 9) => ({
    width: size,
    height: size,
    borderRadius: radius,
  });

  function SideButton(props: { icon: LucideName; label: string; target: Page }) {
    const selected = () => page() === props.target;
    return (
      <View
        testID={`nav-${props.target}`}
        accessibilityLabel={props.label}
        focusable={false}
        onPress={() => navigateToPage(props.target)}
        class="relative mb-1.25 h-9.5 flex-row items-center justify-start rounded-lg pl-3.5 pr-2.5 hover:bg-hover active:opacity-70"
        classList={{ "bg-hover": selected(), "gap-2.5": expanded(), "gap-0": !expanded() }}
      >
        {selected() ? (
          <View
            class="pointer-events-none absolute left-0.75 top-2.75 h-4 w-0.75 rounded-sm"
            style={{ backgroundColor: accentColor() }}
          />
        ) : null}
        <View style={{ width: 20, height: 18, alignItems: "center", justifyContent: "center" }}>
          <LucideIcon
            name={props.icon}
            size={18}
            color={selected() ? accentColor() : secondaryColor()}
          />
        </View>
        {expanded() ? (
          <Text
            style={textStyle(
              14,
              selected() ? textColor() : secondaryColor(),
              selected() ? "600" : "400",
            )}
          >
            {props.label}
          </Text>
        ) : null}
      </View>
    );
  }

  function SongRows(props: { items: SongInfo[]; gallery?: boolean }) {
    return (
      <List
        each={props.items}
        keyExtractor={(song) => song.song_path}
        estimatedItemSize={30}
        contentOffset={props.gallery ? { x: 0, y: config.gallery_viewport_y } : undefined}
        onScroll={
          props.gallery ? (event) => galleryViewportY.set(Math.max(0, event.offset.y)) : undefined
        }
        class="flex-1 bg-page"
        contentStyle={{ paddingBottom: 0 }}
        empty={
          <View class="flex-1 min-h-55 items-center justify-center gap-2">
            <LucideIcon name="search" size={24} color={mutedColor()} />
            <Text style={textStyle(13, mutedColor())}>
              {loading() ? t("Reading your music library…") : t("No songs found in this folder")}
            </Text>
            <Text style={textStyle(12, mutedColor())}>
              {t("Choose a folder with MP3, FLAC, WAV or OGG files")}
            </Text>
          </View>
        }
      >
        {(item) => {
          const isPlaying = () => current()?.song_path === item().song_path;
          const openActions = () => {
            const selected = item();
            void Menu.popup([
              {
                label: t("Edit metadata…"),
                onSelect: () => void beginEdit(selected),
              },
              { type: "separator" },
              { label: t("Delete song…"), onSelect: () => deleteTarget.set(selected) },
            ]);
          };
          return (
            <View
              testID={`song-row-${item().id}`}
              accessibilityLabel={item().song_name}
              onPress={(event) => {
                if (event?.clickCount !== 2) return;
                void playSong(item());
              }}
              onLongPress={openActions}
              onContextMenu={() => openActions()}
              class="h-7.5 flex-row items-center justify-between"
              classList={{ "bg-hover": isPlaying(), "hover:bg-song-hover": !isPlaying() }}
            >
              <View class="w-[33%] justify-center">
                <Text
                  numberOfLines={1}
                  style={{
                    ...textStyle(
                      14,
                      isPlaying() ? accentColor() : textColor(),
                      isPlaying() ? "600" : "400",
                    ),
                    width: "100%",
                    marginLeft: "40%",
                  }}
                >
                  {item().song_name}
                </Text>
              </View>
              <View class="w-[33%] justify-center">
                <Text
                  numberOfLines={1}
                  style={{
                    ...textStyle(14, isPlaying() ? accentColor() : textColor()),
                    width: "100%",
                    marginLeft: "40%",
                  }}
                >
                  {item().singer}
                </Text>
              </View>
              <View class="w-[33%] justify-center">
                <Text
                  style={{
                    ...textStyle(14, isPlaying() ? accentColor() : textColor()),
                    marginLeft: "40%",
                  }}
                >
                  {item().duration}
                </Text>
              </View>
            </View>
          );
        }}
      </List>
    );
  }

  function GalleryPage() {
    return (
      <View testID="screen-0" class="flex-1 min-h-0">
        <View class="h-7.5 flex-row items-center justify-between">
          <View
            onPress={() => {
              if (sortField() === "song_name") ascending.set(!ascending());
              else {
                sortField.set("song_name");
                ascending.set(true);
              }
            }}
            class="w-[33%] h-full flex-row items-center gap-1.25 hover:bg-hover"
          >
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>{t("Title")}</Text>
            {sortField() === "song_name" ? (
              <LucideIcon
                name={ascending() ? "arrowUp" : "arrowDown"}
                size={12}
                color={mutedColor()}
              />
            ) : null}
          </View>
          <View
            onPress={() => {
              if (sortField() === "singer") ascending.set(!ascending());
              else {
                sortField.set("singer");
                ascending.set(true);
              }
            }}
            class="w-[33%] h-full flex-row items-center gap-1.25 hover:bg-hover"
          >
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>{t("Artist")}</Text>
            {sortField() === "singer" ? (
              <LucideIcon
                name={ascending() ? "arrowUp" : "arrowDown"}
                size={12}
                color={mutedColor()}
              />
            ) : null}
          </View>
          <View
            onPress={() => {
              if (sortField() === "duration_secs") ascending.set(!ascending());
              else {
                sortField.set("duration_secs");
                ascending.set(true);
              }
            }}
            class="w-[33%] h-full flex-row items-center gap-1.25 hover:bg-hover"
          >
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>
              {t("Duration")}
            </Text>
            {sortField() === "duration_secs" ? (
              <LucideIcon
                name={ascending() ? "arrowUp" : "arrowDown"}
                size={12}
                color={mutedColor()}
              />
            ) : null}
          </View>
        </View>
        <SongRows items={displayedSongs()} gallery />
      </View>
    );
  }

  function SearchPage() {
    return (
      <View
        testID="screen-1"
        focusable={false}
        onPress={(event) => {
          if ((event?.position.y ?? 0) >= 40) {
            searchInputRef?.blur();
            contentPaneRef?.focus();
          }
        }}
        class="flex-1 min-h-0"
      >
        <View class="h-10 flex-row items-center px-3">
          <View class="ml-0.75 mr-0.5 h-6 flex-1 flex-row items-center rounded-[3px] border-b-2 border-primary">
            <Input
              ref={searchInputRef}
              testID="song-search"
              value={query()}
              placeholder={t("Search songs or artists...")}
              onChange={({ value }) => updateSearch(value)}
              class="h-6 flex-1 bg-transparent px-1 text-[13px]"
              style={{ color: textColor(), fontFamily: appFontFamily }}
            />
            {query() ? (
              <View onPress={() => updateSearch("")} class="mr-1.5 p-0.75">
                <LucideIcon name="x" size={14} color={mutedColor()} />
              </View>
            ) : null}
          </View>
        </View>
        {search() ? (
          <View class="h-6.5 justify-center px-3">
            <Text style={textStyle(12, mutedColor())}>
              {t("{count} results found").replace("{count}", String(searchResults().length))}
            </Text>
          </View>
        ) : null}
        {search() ? (
          <SongRows items={searchResults()} />
        ) : (
          <View class="flex-1 items-center justify-center gap-2">
            <LucideIcon name="search" size={44} color={mutedColor()} />
            <Text style={textStyle(14, mutedColor())}>{t("Type to search your music")}</Text>
          </View>
        )}
      </View>
    );
  }

  function LyricsPage() {
    const panelHeight = signal(470);
    const artworkColumnWidth = signal(0);
    const albumSize = () => Math.min(panelHeight() * 0.4, Math.max(0, artworkColumnWidth() - 32));
    const initialLyricIndex = () => Math.max(0, lyricIndexAt(position()) - 5);
    return (
      <View
        testID="screen-2"
        onLayout={(event) => panelHeight.set(event.bounds.size.height)}
        class="flex-1 min-h-0 flex-row"
      >
        <View
          onLayout={(event) => artworkColumnWidth.set(event.bounds.size.width)}
          class="items-center justify-center gap-1 px-4"
          style={{ width: lyrics().length ? "30%" : "100%" }}
        >
          <View
            class="overflow-hidden rounded-lg"
            style={{
              width: albumSize(),
              height: albumSize(),
              boxShadow: `0 0 8px ${shadowColor()}`,
            }}
          >
            <Image src={art()} class="size-full" style={{ objectFit: "contain" }} />
          </View>
          <View class="w-full">
            <View
              style={{
                width: "100%",
                height: 25,
                alignItems: "center",
                justifyContent: "flex-end",
              }}
            >
              <Text numberOfLines={1} style={textStyle(14, textColor())}>
                {`${t("Artist:")} ${current()?.singer ?? t("Unknown")}`}
              </Text>
            </View>
            <View
              style={{
                width: "100%",
                height: 25,
                alignItems: "center",
                justifyContent: "flex-end",
              }}
            >
              <Text numberOfLines={1} style={textStyle(14, textColor())}>
                {`${t("Title:")} ${current()?.song_name ?? t("No song")}`}
              </Text>
            </View>
          </View>
        </View>
        {lyrics().length ? (
          <List
            testID="lyrics-list"
            ref={lyricListRef}
            each={lyrics()}
            keyExtractor={(line, index) => `${line.time}:${index}`}
            estimatedItemSize={40}
            initialScrollIndex={initialLyricIndex()}
            onScroll={(event) => observeLyricsScroll(event.offset.y)}
            onScrollDragStart={noteManualLyricsScroll}
            class="w-[70%] flex-1"
            contentClass="px-3 pt-1 pb-3"
          >
            {(line, index) => {
              const active = () => lyricIndexAt(position()) === index();
              return (
                <View
                  testID={active() ? "lyric-active" : `lyric-${index()}`}
                  onPress={() => seekTo(line().time)}
                  class="h-10 items-center justify-center rounded-[5px] hover:bg-song-hover"
                >
                  <Text
                    numberOfLines={1}
                    style={textStyle(
                      active() ? 20 : 17,
                      active() ? accentColor() : textColor(),
                      active() ? "600" : "400",
                    )}
                  >
                    {line().text}
                  </Text>
                </View>
              );
            }}
          </List>
        ) : null}
      </View>
    );
  }

  function SettingsPage() {
    const cardClass = "gap-2.5 rounded-lg border border-border bg-panel p-4";
    const changeFollowSystem = () => {
      const next = !followSystem();
      if (next) {
        followSystem.set(true);
        setColorScheme("system");
      } else {
        const currentScheme = colorScheme();
        followSystem.set(false);
        setColorScheme(currentScheme);
      }
    };
    const changeSpectrum = () => {
      const next = !showSpectrum();
      showSpectrum.set(next);
      Music.setSpectrumEnabled(next);
    };
    const checkbox = (checked: boolean, label: string, onPress: () => void) => (
      <NshCheckbox
        checked={checked}
        accessibilityLabel={label}
        onChange={onPress}
        class="items-center justify-center"
        indicatorClass="size-4.5 items-center justify-center rounded-sm border border-checkbox-border bg-transparent checked:border-primary checked:bg-primary"
        indicator={
          checked ? <LucideIcon name="check" size={15} color={token("primary-foreground")} /> : null
        }
      />
    );

    return (
      <View testID="screen-3" class="flex-1 min-h-0">
        <Scroll testID="settings-scroll" class="flex-1" contentClass="p-6 gap-5">
          <View class={cardClass}>
            <Text style={textStyle(12, mutedColor(), "600")}>{t("Playback")}</Text>
            <View class="h-px bg-divider" />
            <View class="gap-1.5">
              <Text style={textStyle(13, secondaryColor())}>{t("Music directory")}</Text>
              <View class="flex-row items-center gap-2">
                <View class="h-8 min-w-0 flex-1 justify-center rounded border border-input-border bg-raised px-3">
                  <Text numberOfLines={1} style={textStyle(13, secondaryColor())}>
                    {directory()}
                  </Text>
                </View>
                <View
                  onPress={() => void chooseDirectory()}
                  class="h-8 flex-row items-center justify-center rounded bg-button px-3"
                >
                  <Text style={textStyle(14, textColor(), "500")}>{t("Browse…")}</Text>
                </View>
              </View>
            </View>
            <View class="flex-row items-center gap-2.5">
              {checkbox(showSpectrum(), t("Spectrum display"), changeSpectrum)}
              <Text onPress={changeSpectrum} style={textStyle(14, textColor())}>
                {t("Spectrum display")}
              </Text>
            </View>
          </View>
          <View class={cardClass}>
            <Text style={textStyle(12, mutedColor(), "600")}>{t("Window")}</Text>
            <View class="h-px bg-divider" />
            <View class="flex-row items-center gap-2.5">
              {checkbox(closeToTray(), t("Keep in system tray when closing"), () =>
                closeToTray.set(!closeToTray()),
              )}
              <Text
                onPress={() => closeToTray.set(!closeToTray())}
                style={textStyle(14, textColor())}
              >
                {t("Keep in system tray when closing")}
              </Text>
            </View>
          </View>
          <View class={cardClass}>
            <Text style={textStyle(12, mutedColor(), "600")}>{t("Appearance")}</Text>
            <View class="h-px bg-divider" />
            <View class="flex-row items-center gap-2.5">
              {checkbox(followSystem(), t("Follow system theme"), changeFollowSystem)}
              <Text onPress={changeFollowSystem} style={textStyle(14, textColor())}>
                {t("Follow system theme")}
              </Text>
            </View>
            <View class="gap-1.5">
              <Text style={textStyle(13, secondaryColor())}>{t("Language")}</Text>
              <View class="flex-row gap-2">
                {[
                  { code: "en", label: languageName("en") },
                  { code: "zh", label: languageName("zh") },
                  { code: "es", label: languageName("es") },
                  { code: "fr", label: languageName("fr") },
                  { code: "de", label: languageName("de") },
                  { code: "ru", label: languageName("ru") },
                ].map((option) => {
                  return (
                    <View
                      testID={`language-${option.code}`}
                      onPress={() => {
                        language.set(option.code as Language);
                      }}
                      class="h-6.5 min-w-0 flex-1 items-center justify-center rounded border border-input-border bg-hover"
                      classList={{
                        "border-primary": language() === option.code,
                        "bg-primary": language() === option.code,
                      }}
                    >
                      <Text
                        numberOfLines={1}
                        classList={{
                          "text-primary-foreground": language() === option.code,
                          "text-foreground": language() !== option.code,
                        }}
                        style={{ fontSize: 14, fontFamily: appFontFamily }}
                      >
                        {option.label}
                      </Text>
                    </View>
                  );
                })}
              </View>
            </View>
          </View>
        </Scroll>
      </View>
    );
  }

  function AboutPage() {
    return (
      <View testID="screen-4" class="flex-1 items-center justify-center gap-2.5">
        <Image src={cover} style={{ width: 120, height: 120 }} />
        <Text numberOfLines={2} style={{ ...textStyle(14, textColor()), textAlign: "center" }}>
          {`Zeedle\n${t("Music at the speed of a blink, designed for pure simplicity")}`}
        </Text>
        <View
          style={{ flexDirection: "row", alignItems: "center", justifyContent: "center", gap: 4 }}
        >
          <Text style={textStyle(14, textColor())}>{t("Built by")}</Text>
          <View
            role="link"
            accessibilityLabel="Rodio"
            onPress={() => void openExternalUrl("https://github.com/RustAudio/rodio")}
          >
            <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>
              Rodio
            </Text>
          </View>
          <Text style={textStyle(14, textColor())}>{t("and")}</Text>
          <View
            testID="about-nsh-link"
            role="link"
            accessibilityLabel="NSH"
            onPress={() => void openExternalUrl("https://nsh.dev")}
          >
            <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>
              NSH
            </Text>
          </View>
        </View>
        <Text numberOfLines={2} style={{ ...textStyle(14, textColor()), textAlign: "center" }}>
          {t("Author:")} Jordan Haidee{"\n"}
          {t("Version:")} 0.2.7
        </Text>
        <View
          role="link"
          accessibilityLabel="Github"
          onPress={() => void openExternalUrl("https://github.com/Jordan-Haidee/Zeedle")}
        >
          <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>
            Github
          </Text>
        </View>
      </View>
    );
  }

  function PlayerPanel() {
    return (
      <View class="h-18 min-h-18 bg-page">
        <View
          testID="progress-seek"
          hitSlop={{ top: 8, bottom: 8 }}
          onPress={(event) => seekTo(progressAtX(event?.position.x ?? 0))}
          onLayout={(event) => trackWidth.set(event.bounds.size.width)}
          class="mx-2.5 h-1 justify-center bg-track"
        >
          <ProgressSlider
            position={position}
            duration={duration}
            label={t("Playback progress")}
            onChange={(next) => {
              seeking.set(true);
              position.set(next);
            }}
            onChangeEnd={(next) => {
              seekTo(next);
              seeking.set(false);
            }}
          />
        </View>
        <View class="h-17 flex-row items-center">
          <View class="w-[35%] flex-row items-center gap-0.75" onPress={() => navigateToPage(2)}>
            <View class="w-[20%] items-center justify-center">
              <View class="size-10 overflow-hidden rounded">
                <Image src={art()} class="size-10" />
              </View>
            </View>
            <View class="w-[80%]">
              <Text
                numberOfLines={1}
                style={{ ...textStyle(14, textColor()), textAlign: "center" }}
              >
                {current()
                  ? `${current()!.song_name} - ${current()!.singer}`
                  : t("No song selected")}
              </Text>
            </View>
          </View>
          <View class="w-[6%] flex-row items-center justify-center">
            <View
              testID="control-repeat"
              accessibilityLabel={t("Repeat current track")}
              accessibilityState={{ selected: mode() !== "random" }}
              onPress={() => changeMode(mode() === "order" ? "repeat" : "order")}
              class="items-center justify-center"
              style={controlStyle(20)}
            >
              <LucideIcon
                name={mode() === "repeat" ? "repeat2" : "repeat"}
                size={18}
                color={mode() !== "random" ? accentColor() : secondaryColor()}
              />
            </View>
          </View>
          <View class="w-[6%] flex-row items-center justify-center">
            <PlayerTransportButton
              testID="control-previous"
              label={t("Previous track")}
              name="skipBack"
              color={secondaryColor()}
              onPress={playPrevious}
              radius={8}
            />
          </View>
          <View class="w-[6%] flex-row items-center justify-center">
            <PlayerTransportButton
              testID="control-play-pause"
              label={paused() ? t("Play") : t("Pause")}
              name={paused() ? "play" : "pause"}
              color={accentColor()}
              onPress={togglePlay}
            />
          </View>
          <View class="w-[6%] flex-row items-center justify-center">
            <PlayerTransportButton
              testID="control-next"
              label={t("Next track")}
              name="skipForward"
              color={secondaryColor()}
              onPress={playNext}
              radius={8}
            />
          </View>
          <View
            style={{
              width: "6%",
              flexDirection: "row",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <View
              testID="control-shuffle"
              accessibilityLabel={t("Shuffle")}
              accessibilityState={{ selected: mode() === "random" }}
              onPress={() => changeMode("random")}
              class="items-center justify-center"
              style={controlStyle(20)}
            >
              <LucideIcon
                name="shuffle"
                size={18}
                color={mode() === "random" ? accentColor() : secondaryColor()}
              />
            </View>
          </View>
          <View class="w-[25%] items-center justify-center">
            <Text style={textStyle(14, textColor())}>
              {timeLabel(position())} / {timeLabel(duration())}
            </Text>
          </View>
          <View class="w-[10%] items-center justify-center">
            <VolumeControl
              volume={volume}
              volumeLabel={t("Volume")}
              onVolumeChange={(next) => {
                volume.set(next);
                Music.setVolume(next);
              }}
            />
          </View>
        </View>
      </View>
    );
  }

  return (
    <View class="min-h-125 min-w-212.5 flex-1 flex-col overflow-hidden border border-border bg-page">
      <TitleBar class="bg-panel">
        <View class="h-10 flex-1 flex-row items-center pl-2.5">
          <View class="mr-1.5 size-4.5 overflow-hidden rounded">
            <Image src={cover} class="size-4.5" />
          </View>
          <Text style={textStyle(13, textColor(), "600")}>Zeedle</Text>
          <View class="flex-1" />
          <View
            testID="theme-toggle"
            accessibilityLabel={t("Toggle theme")}
            disabled={followSystem()}
            disabledStyle={{ opacity: 0.4 }}
            class="h-10 w-10.5 items-center justify-center hover:bg-hover"
            onPress={() => {
              if (followSystem()) return;
              const next = !light();
              setColorScheme(next ? "light" : "dark");
            }}
          >
            <LucideIcon name={light() ? "sun" : "moon"} size={18} color={secondaryColor()} />
          </View>
          {win.titleBar().right === 0 ? (
            <View style={{ flexDirection: "row", alignItems: "center" }}>
              <View class="h-4 w-px bg-divider" />
              <View
                onPress={() => win.minimize()}
                class="h-10 w-10.5 items-center justify-center hover:bg-hover"
              >
                <LucideIcon name="minus" size={18} color={secondaryColor()} />
              </View>
              <View
                onPress={() =>
                  win.setMode(win.state()?.mode === "maximized" ? "normal" : "maximized")
                }
                class="h-10 w-10.5 items-center justify-center hover:bg-hover"
              >
                <LucideIcon name="square" size={12} color={secondaryColor()} />
              </View>
              <View
                testID="window-close"
                accessibilityLabel={t("Hide window")}
                onPress={handleWindowClose}
                class="h-10 w-10.5 items-center justify-center"
                hoverStyle={{ backgroundColor: "#e81123" }}
              >
                <LucideIcon name="x" size={14} color={secondaryColor()} />
              </View>
            </View>
          ) : null}
        </View>
      </TitleBar>

      <View style={{ flex: 1, minHeight: 0, flexDirection: "row" }}>
        <View class="bg-panel px-1.5 pt-2.25 pb-2.5" style={{ width: expanded() ? 200 : 54 }}>
          <View
            testID="sidebar-toggle"
            focusable={false}
            onPress={() => {
              LayoutAnimation.configureNext({
                duration: 200,
                update: { type: "easeOut" },
                create: { type: "easeOut", property: "opacity", duration: 120 },
                delete: { type: "easeOut", property: "opacity", duration: 120 },
              });
              expanded.set(!expanded());
            }}
            class="mb-2.5 h-9 flex-row items-center justify-start gap-2.5 rounded-lg pl-3.5 pr-2.5 hover:bg-hover"
          >
            <View class="h-4.5 w-5 items-center justify-center">
              <LucideIcon
                name={expanded() ? "panelLeftClose" : "panelLeftOpen"}
                size={18}
                color={secondaryColor()}
              />
            </View>
          </View>
          <View class="h-1.25" />
          <SideButton icon="listMusic" label={t("Gallery")} target={0} />
          <SideButton icon="search" label={t("Search")} target={1} />
          <SideButton icon="captions" label={t("Lyrics")} target={2} />
          <View class="flex-1" />
          <View class="mx-px mb-2 h-px bg-border" />
          <SideButton icon="settings" label={t("Settings")} target={3} />
          <SideButton icon="circleHelp" label={t("About")} target={4} />
        </View>

        <View
          ref={contentPaneRef}
          focusable
          accessibilityLabel={t("Main content")}
          class="relative flex-1 min-h-0 min-w-0 flex-col bg-panel"
        >
          <View class="flex-1 min-h-0 overflow-hidden bg-page" style={contentCornerStyle}>
            {page() === 0 ? <GalleryPage /> : null}
            {page() === 1 ? <SearchPage /> : null}
            {page() === 2 ? <LyricsPage /> : null}
            {page() === 3 ? <SettingsPage /> : null}
            {page() === 4 ? <AboutPage /> : null}
          </View>
          {page() === 0 ? <PlayerPanel /> : null}
          {showSpectrum() ? (
            <View class="pointer-events-none absolute bottom-0 left-0 right-0 z-5 h-15 items-center justify-end">
              <View class="h-15 w-1/2 flex-row items-end opacity-30">
                {spectrumBars().map((bar) => (
                  <View
                    class="flex-1 rounded-[1px] bg-primary"
                    style={{ height: Math.max(0, Math.min(1, bar)) * 60 }}
                  />
                ))}
              </View>
            </View>
          ) : null}
        </View>
      </View>

      {playerError() ? (
        <View
          onPress={() => playerError.set("")}
          style={{
            position: "absolute",
            right: 15,
            bottom: page() === 0 ? 82 : 12,
            width: "80%",
            maxWidth: 334,
            paddingHorizontal: 12,
            paddingVertical: 9,
            flexDirection: "row",
            alignItems: "center",
            gap: 8,
            backgroundColor: dangerSurfaceColor(),
            borderRadius: 7,
            borderWidth: 1,
            borderColor: dangerColor(),
          }}
        >
          <LucideIcon name="circleAlert" size={16} color={dangerColor()} />
          <Text
            numberOfLines={2}
            style={{ ...textStyle(11, dangerTextColor()), flex: 1, minWidth: 0 }}
          >
            {playerError()}
          </Text>
        </View>
      ) : null}

      {editingSong() ? (
        <View class="absolute inset-0 z-20 items-center justify-center p-2.5">
          <View class="h-112.5 w-130 max-h-[96%] max-w-[96%] gap-[7.5px] rounded-lg border border-border bg-page px-4 pt-3.5 pb-4">
            <Text style={textStyle(18, textColor(), "600")}>{t("Edit song metadata")}</Text>
            <View style={{ gap: 4 }}>
              <Text style={textStyle(14, secondaryColor())}>{t("Song title")}</Text>
              <Input
                testID="edit-title"
                value={editTitle()}
                onChange={({ value }) => editTitle.set(value)}
                class="h-8 rounded border border-border bg-raised px-2.5 text-sm text-foreground"
              />
            </View>
            <View style={{ flexDirection: "row", gap: 12 }}>
              <View style={{ flex: 1, gap: 4 }}>
                <Text style={textStyle(14, secondaryColor())}>{t("Artist")}</Text>
                <Input
                  testID="edit-artist"
                  value={editArtist()}
                  onChange={({ value }) => editArtist.set(value)}
                  class="h-8 rounded border border-border bg-raised px-2.5 text-sm text-foreground"
                />
              </View>
              <View style={{ flex: 1, gap: 4 }}>
                <Text style={textStyle(14, secondaryColor())}>{t("Album")}</Text>
                <Input
                  testID="edit-album"
                  value={editAlbum()}
                  onChange={({ value }) => editAlbum.set(value)}
                  class="h-8 rounded border border-border bg-raised px-2.5 text-sm text-foreground"
                />
              </View>
            </View>
            <View
              style={{ height: 200, minHeight: 200, marginTop: -1, flexDirection: "row", gap: 10 }}
            >
              <View class="min-w-0 flex-1 gap-1.5 rounded-lg border border-border bg-panel pt-2 pb-2.5 pl-2 pr-2.5">
                <Text style={textStyle(14, textColor(), "600")}>{t("Album cover")}</Text>
                <View class="-mt-px min-h-0 flex-row items-start">
                  <Image src={editCoverPreview()} class="size-25" />
                </View>
                <View
                  onPress={() => void chooseEditCover()}
                  class="h-8 items-center justify-center rounded bg-raised"
                >
                  <Text style={textStyle(13, textColor())}>{t("Select cover image")}</Text>
                </View>
                <Text style={textStyle(11, mutedColor())}>
                  {editCoverPath() ? editCoverName() : t("Existing cover will be kept")}
                </Text>
              </View>
              <View class="min-w-0 flex-1 gap-1.5 rounded-lg border border-border bg-panel px-2.5 pt-2 pb-2.5">
                <Text style={textStyle(14, textColor(), "600")}>{t("Lyrics")}</Text>
                <Scroll class="-mt-px h-25 min-h-25 rounded bg-raised p-2">
                  {(editLyricsPreview() || t("No embedded lyrics")).split(/\r?\n/).map((line) => (
                    <Text
                      numberOfLines={1}
                      style={{ ...textStyle(11, textColor()), lineHeight: 13 }}
                    >
                      {line}
                    </Text>
                  ))}
                </Scroll>
                <View
                  onPress={() => void chooseEditLyrics()}
                  class="h-8 items-center justify-center rounded bg-raised"
                >
                  <Text style={textStyle(13, textColor())}>{t("Select LRC file")}</Text>
                </View>
                <Text style={textStyle(11, mutedColor())}>
                  {editLyricsPath() ? editLyricsName() : t("Existing lyrics will be kept")}
                </Text>
              </View>
            </View>
            {editError() ? <Text style={textStyle(11, dangerColor())}>{editError()}</Text> : null}
            <View class="flex-1" />
            <View class="h-px bg-divider" />
            <View class="flex-row justify-end gap-2">
              <View
                testID="edit-cancel"
                onPress={() => editingSong.set(null)}
                class="h-8 w-17 items-center justify-center rounded bg-raised"
              >
                <Text style={textStyle(13, textColor())}>{t("Cancel")}</Text>
              </View>
              <View
                onPress={() => void saveEdit()}
                class="h-8 w-13.75 items-center justify-center rounded bg-button"
                style={{ opacity: editSaving() ? 0.65 : 1 }}
              >
                <Text style={textStyle(13, textColor(), "500")}>
                  {editSaving() ? t("Saving…") : t("Save")}
                </Text>
              </View>
            </View>
          </View>
        </View>
      ) : null}

      {deleteTarget() ? (
        <View class="absolute inset-0 z-21 items-center justify-center p-5">
          <View class="w-92.5 max-w-full gap-2.75 rounded-lg border border-border bg-page p-5">
            <Text style={textStyle(17, textColor(), "600")}>{t("Delete this song?")}</Text>
            <Text style={textStyle(13, textColor(), "500")}>{deleteTarget()?.song_name}</Text>
            <Text numberOfLines={2} style={textStyle(11, secondaryColor())}>
              {t("The audio file will be removed from your device.")}
            </Text>
            <View
              style={{ flexDirection: "row", justifyContent: "flex-end", gap: 8, marginTop: 5 }}
            >
              <View
                testID="delete-cancel"
                onPress={() => deleteTarget.set(null)}
                class="rounded bg-raised px-3.25 py-2"
              >
                <Text style={textStyle(12, secondaryColor(), "500")}>{t("Cancel")}</Text>
              </View>
              <View onPress={() => void confirmDelete()} class="rounded bg-danger px-3.5 py-2">
                <Text style={textStyle(12, token("destructive-foreground"), "600")}>
                  {t("Delete song")}
                </Text>
              </View>
            </View>
          </View>
        </View>
      ) : null}
    </View>
  );
}

export function App() {
  return (
    <Window
      id="main"
      title="Zeedle"
      titleBar={{ style: "hidden", height: 40 }}
      minSize={{ width: 850, height: 500 }}
      maxSize={null}
      closeOnPress={false}
    >
      {AppWindow}
    </Window>
  );
}
