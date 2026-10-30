import { Image, LayoutAnimation, List, Scroll, Storage, Text as NshText, View, colorScheme, keymap, mount, setColorScheme, signal, split, token, windowSize, type YNode } from "nsh";
import { Input } from "nsh/base";
import { Menu, titleBar, Window } from "nsh/desktop";
import { Music, WindowConstraints } from "zeedle-nsh/native";
import { LucideIcon, type LucideName } from "./icons";
import cover from "./assets/cover.svg";
import { translate, type Language } from "./translations";

type Page = 0 | 1 | 2 | 3 | 4;
type SortField = "song_name" | "singer" | "duration_secs";
type PlayMode = "order" | "repeat" | "random";
type SongInfo = { id: number; song_name: string; singer: string; duration: string; duration_secs: number; song_path: string };
type LyricItem = { time: number; duration: number; text: string };
type PlaybackState = { position: number; duration: number; paused: boolean; volume: number; error: string; spectrum: number[] };

type NshTextProps = Parameters<typeof NshText>[0];
function Text(props: NshTextProps) {
  const [, rest] = split(props, ["numberOfLines", "ellipsizeMode", "children"] as const);
  return (
    <NshText {...rest} numberOfLines={props.numberOfLines ?? 1} ellipsizeMode={props.ellipsizeMode ?? "tail"}>
      {props.children}
    </NshText>
  );
}

function timeLabel(seconds: number) {
  const value = Math.max(0, Math.floor(seconds || 0));
  return `${String(Math.floor(value / 60)).padStart(2, "0")}:${String(value % 60).padStart(2, "0")}`;
}

function App() {
  const page = signal<Page>(0);
  const expanded = signal(false);
  const light = () => colorScheme() === "light";
  const followSystem = signal(true);
  const language = signal<Language>("en");
  const directory = signal(Music.defaultDirectory());
  const songs = signal<SongInfo[]>([]);
  const current = signal<SongInfo | null>(null);
  const lyrics = signal<LyricItem[]>([]);
  const art = signal<string | typeof cover>(cover);
  const query = signal("");
  const search = signal("");
  const searchResults = signal<SongInfo[]>([]);
  const loading = signal(false);
  const sortField = signal<SortField>("song_name");
  const ascending = signal(true);
  const mode = signal<PlayMode>("order");
  const position = signal(0);
  const duration = signal(0);
  const paused = signal(true);
  const playerLoaded = signal(false);
  const volume = signal(0.8);
  const showSpectrum = signal(true);
  const spectrumBars = signal<number[]>(Array.from({ length: 48 }, () => 0));
  const playerError = signal("");
  const trackWidth = signal(1);
  const history = signal<string[]>([]);
  const historyIndex = signal(-1);
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
  let lyricListRef!: YNode;
  let lastAutoScrolledLyric = -1;
  let manualLyricsScrollUntil = 0;
  let ignoreProgrammaticLyricsScrollUntil = 0;
  let lastLyricsScrollOffset: number | null = null;
  let handledEnd = false;

  const t = (english: string, chinese: string) => translate(language(), english, chinese);
  async function openExternalUrl(url: string) {
    const error = await Music.openUrl(url);
    if (error) playerError.set(error);
  }

  Window.setTitle("Zeedle");
  Window.setTitleBar("hidden", 40);
  WindowConstraints.setMinimumSize(850, 500);
  const initialWindowSize = windowSize();
  const hostDefaultSize =
    (initialWindowSize.width === 1200 && initialWindowSize.height === 800) ||
    (initialWindowSize.width === 390 && initialWindowSize.height === 844) ||
    // Packaged GTK starts at the app's minimum content size when no host size is set.
    (initialWindowSize.width === 850 && initialWindowSize.height === 500);
  const initialWidth = hostDefaultSize ? 900 : Math.max(initialWindowSize.width, 850);
  const initialHeight = hostDefaultSize ? 510 : Math.max(initialWindowSize.height, 500);
  setTimeout(() => WindowConstraints.configure(initialWidth, initialHeight, 850, 500), 150);

  const displayedSongs = () => {
    const items = [...songs()];
    const field = sortField();
    const dir = ascending() ? 1 : -1;
    return items.sort((a, b) => {
      const left = a[field];
      const right = b[field];
      return (typeof left === "number" && typeof right === "number"
        ? left - right
        : String(left).localeCompare(String(right), undefined, { sensitivity: "base" })) * dir;
    });
  };

  const searchSongs = (rawQuery = search()) => {
    const term = rawQuery.trim().toLocaleLowerCase();
    if (!term) return [];
    return songs()
      .map((song, index) => {
        const title = song.song_name.toLocaleLowerCase();
        const artist = song.singer.toLocaleLowerCase();
        const score = (title === term ? 100 : title.startsWith(term) ? 80 : title.includes(term) ? 60 : 0)
          + (artist === term ? 50 : artist.startsWith(term) ? 40 : artist.includes(term) ? 20 : 0);
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
      await Storage.setItem("zeedle.music-directory", path);
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
      const active = result.find((song) => song.song_path === current()?.song_path) ?? result[0];
      if (!current() || !result.some((song) => song.song_path === current()?.song_path)) {
        Music.stop();
        playerLoaded.set(true);
        current.set(active);
        position.set(0);
        duration.set(active.duration_secs);
        paused.set(true);
        lastAutoScrolledLyric = -1;
        Music.load(active.song_path);
        const [lyricsJson, nextArt] = await Promise.all([Music.lyrics(active.song_path), Music.cover(active.song_path)]);
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
    const [lyricsJson, nextArt] = await Promise.all([Music.lyrics(song.song_path), Music.cover(song.song_path)]);
    const nextLyrics = JSON.parse(lyricsJson) as LyricItem[];
    if (current()?.song_path === song.song_path) {
      lyrics.set(nextLyrics);
      art.set(nextArt || cover);
    }
  }

  function playNext() {
    const list = displayedSongs();
    if (!list.length) return;
    if (mode() === "repeat" && current()) {
      void playSong(current()!, false);
      return;
    }
    const index = list.findIndex((song) => song.song_path === current()?.song_path);
    const nextIndex = mode() === "random"
      ? Math.floor(Math.random() * list.length)
      : (index + 1 + list.length) % list.length;
    void playSong(list[nextIndex]);
  }

  function playPrevious() {
    const previous = historyIndex() - 1;
    if (previous >= 0) {
      historyIndex.set(previous);
      const song = songs().find((item) => item.song_path === history()[previous]);
      if (song) void playSong(song, false);
      return;
    }
    const list = displayedSongs();
    if (!list.length) return;
    const index = list.findIndex((song) => song.song_path === current()?.song_path);
    void playSong(list[(index - 1 + list.length) % list.length]);
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
    void Storage.setItem("zeedle.play-mode", next);
  }

  function seekTo(seconds: number) {
    const next = Math.max(0, Math.min(duration(), seconds));
    position.set(next);
    Music.seek(next);
  }

  function noteManualLyricsScroll() {
    manualLyricsScrollUntil = Date.now() + 5000;
    lastAutoScrolledLyric = -1;
  }

  function observeLyricsScroll(offset: number) {
    const changed = lastLyricsScrollOffset !== null && Math.abs(offset - lastLyricsScrollOffset) > 2;
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

  function progressAtX(x: number) {
    const width = trackWidth();
    return width > 0 && duration() > 0
      ? Math.max(0, Math.min(1, x / width)) * duration()
      : 0;
  }

  function seekBy(seconds: number) {
    if (page() === 2 && lyrics().length) {
      const target = seconds < 0
        ? [...lyrics()].reverse().find((line) => line.time < position() - 0.5)?.time ?? 0
        : lyrics().find((line) => line.time > position() + 0.5)?.time ?? position();
      seekTo(target);
    } else {
      seekTo(position() + seconds);
    }
  }

  function navigateToPage(target: Page) {
    if (target === 2) {
      lastAutoScrolledLyric = -1;
      lastLyricsScrollOffset = null;
      ignoreProgrammaticLyricsScrollUntil = Date.now() + 500;
    }
    page.set(target);
  }

  keymap.bind({
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
  }, { when: "notEditing" });

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
      const metadata = JSON.parse(await Music.editorMetadata(song.song_path)) as { title?: string; artist?: string; album?: string; cover?: string; lyrics?: string };
      if (editingSong()?.song_path === song.song_path) {
        editTitle.set(metadata.title || song.song_name);
        editArtist.set(metadata.artist || song.singer);
        editAlbum.set(metadata.album || "");
        editCoverPreview.set(metadata.cover || cover);
        editCoverName.set(metadata.cover ? t("Embedded artwork", "内嵌封面") : t("No cover image", "没有封面图片"));
        editLyricsPreview.set(metadata.lyrics || "");
        editLyricsName.set(metadata.lyrics ? t("Embedded lyrics", "内嵌歌词") : t("No embedded lyrics", "没有内嵌歌词"));
      }
    } catch (error) {
      editError.set(String(error));
    }
  }

  async function chooseEditCover() {
    const path = await Music.chooseCover();
    if (!path) return;
    const preview = await Music.coverFile(path);
    if (!preview) { editError.set(t("Could not read that image file.", "无法读取该图片文件。")); return; }
    editCoverPath.set(path);
    editCoverName.set(path.split(/[\\/]/).pop() || path);
    editCoverPreview.set(preview);
    editError.set("");
  }

  async function chooseEditLyrics() {
    const path = await Music.chooseLyrics();
    if (!path) return;
    const text = await Music.lyricsFile(path);
    if (!text) { editError.set(t("Could not read that LRC file.", "无法读取该 LRC 文件。")); return; }
    editLyricsPath.set(path);
    editLyricsName.set(path.split(/[\\/]/).pop() || path);
    editLyricsPreview.set(text);
    editError.set("");
  }

  async function saveEdit() {
    const song = editingSong();
    if (!song || editSaving()) return;
    if (!editTitle().trim() || !editArtist().trim()) {
      editError.set(t("Title and artist are required.", "歌曲名和艺术家必填。"));
      return;
    }
    editSaving.set(true);
    editError.set("");
    try {
      const error = await Music.updateMetadata(song.song_path, editTitle(), editArtist(), editAlbum(), editCoverPath(), editLyricsPath());
      if (error) { editError.set(error); return; }
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
    if (error) { playerError.set(error); return; }
    await refreshSongs();
  }

  void (async () => {
    const [savedDirectory, savedTheme, savedVolume, savedMode, savedSpectrum, savedLanguage] = await Promise.all([
      Storage.getItem("zeedle.music-directory"),
      Storage.getItem("zeedle.light-theme"),
      Storage.getItem("zeedle.volume"),
      Storage.getItem("zeedle.play-mode"),
      Storage.getItem("zeedle.show-spectrum"),
      Storage.getItem("zeedle.language"),
    ]);
    if (savedTheme !== null) { followSystem.set(false); setColorScheme(savedTheme === "true" ? "light" : "dark"); }
    else { followSystem.set(true); setColorScheme("system"); }
    if (savedVolume !== null) {
      const parsed = Number(savedVolume);
      if (Number.isFinite(parsed)) volume.set(Math.max(0, Math.min(1, parsed)));
    }
    if (savedMode === "repeat" || savedMode === "random") mode.set(savedMode);
    Music.setVolume(volume());
    if (savedSpectrum !== null) showSpectrum.set(savedSpectrum === "true");
    if (savedLanguage === "zh" || savedLanguage === "es" || savedLanguage === "fr" || savedLanguage === "de" || savedLanguage === "ru") language.set(savedLanguage);
    Music.setSpectrumEnabled(showSpectrum());
    await refreshSongs(savedDirectory || directory());
  })().catch((error) => playerError.set(String(error))).finally(() => {
    setTimeout(() => Music.finishStartup?.(), 150);
  });

  setInterval(() => {
    const state = JSON.parse(Music.state()) as PlaybackState;
    position.set(state.position);
    if (state.duration > 0) duration.set(state.duration);
    if (!pausePending()) paused.set(state.paused);
    autoScrollLyrics(state.position);
    if (state.error) playerError.set(state.error);
    volume.set(state.volume);
    spectrumBars.set(state.spectrum);
    if (!state.paused && state.duration > 0 && state.position >= state.duration - 0.12 && !handledEnd) {
      handledEnd = true;
      playNext();
    }
  }, 120);

  const textColor = () => token("foreground");
  const secondaryColor = () => token("text-secondary");
  const mutedColor = () => token("text-tertiary");
  const panelColor = () => token("panel");
  const raisedColor = () => token("raised");
  const lineColor = () => token("border");
  const pageColor = () => token("page");
  const hoverColor = () => token("hover");
  const songHoverColor = () => token("song-hover");
  const pressedColor = () => token("pressed");
  const accentColor = () => token("primary");
  const primaryForegroundColor = () => token("primary-foreground");
  const inputBorderColor = () => token("input-border");
  const dividerColor = () => token("divider");
  const buttonColor = () => token("button");
  const checkboxBorderColor = () => token("checkbox-border");
  const dangerColor = () => token("danger");
  const dangerSurfaceColor = () => token("danger-surface");
  const dangerTextColor = () => token("danger-text");
  const shadowColor = () => token("shadow");
  const trackColor = () => token("track");
  const trackThumbBorderColor = () => token("track-thumb-border");

  const textStyle = (size = 13, color = textColor(), weight: "400" | "500" | "600" | "700" = "400") => ({
    fontSize: size,
    color,
    fontWeight: weight,
    fontFamily: "Microsoft YaHei UI",
  });

  const controlStyle = (size = 36, radius = 9) => ({
    position: "relative" as const,
    width: size,
    height: size,
    borderRadius: radius,
    alignItems: "center" as const,
    justifyContent: "center" as const,
    backgroundColor: "transparent",
    transitionProperty: "backgroundColor",
    transitionDuration: 120,
    transitionTimingFunction: "ease-out" as const,
  });

  function PlayerTransportIcon(props: { name: LucideName; color: string; visible?: boolean }) {
    return (
      <View pointerEvents="none" style={{ position: "absolute", left: 5, top: 5, width: 26, height: 26, opacity: props.visible === false ? 0 : 1, alignItems: "center", justifyContent: "center" }}>
        <Image src={Music.transportIcon(props.name, props.color)} resizeMode="contain" style={{ width: 26, height: 26 }} />
      </View>
    );
  }

  function PlayerTransportButton(props: { testID: string; label: string; name: LucideName; color: string; onPress: () => void; radius?: number }) {
    const hovered = signal(false);
    return (
      <View
        testID={props.testID}
        accessibilityLabel={props.label}
        onPress={props.onPress}
        onHoverIn={() => hovered.set(true)}
        onHoverOut={() => hovered.set(false)}
        activeStyle={{ backgroundColor: pressedColor() }}
        style={{ ...controlStyle(36, props.radius ?? 9), backgroundColor: hovered() ? hoverColor() : "transparent" }}
      >
        {props.name === "play" || props.name === "pause" ? <>
          <PlayerTransportIcon name="play" color={props.color} visible={props.name === "play"} />
          <PlayerTransportIcon name="pause" color={props.color} visible={props.name === "pause"} />
        </> : <PlayerTransportIcon name={props.name} color={props.color} />}
      </View>
    );
  }

  function VolumeControl() {
    const hovered = signal(false);
    const setVolumeAt = (y: number) => {
      const next = Math.max(0, Math.min(1, 1 - y / 55));
      volume.set(next);
      Music.setVolume(next);
      void Storage.setItem("zeedle.volume", String(next));
    };
    return (
      <View
        style={{ width: "100%", height: 68, position: "relative", alignItems: "center", justifyContent: "center" }}
        onHoverIn={() => hovered.set(true)}
        onHoverOut={() => hovered.set(false)}
      >
        {hovered() ? (
          <View
            accessibilityLabel={t("Volume", "音量")}
            accessibilityValue={{ min: 0, max: 1, now: volume() }}
            role="adjustable"
            onPress={(event) => setVolumeAt(event?.position.y ?? 27.5)}
            style={{ position: "absolute", right: 13, top: 6, width: 20, height: 55 }}
          >
            <View pointerEvents="none" style={{ position: "absolute", left: 8, top: 0, width: 4, height: 55, borderRadius: 2, backgroundColor: trackColor() }}>
              <View style={{ position: "absolute", left: 0, bottom: 0, width: 4, height: 55 * volume(), borderRadius: 2, backgroundColor: accentColor() }} />
              <View style={{ position: "absolute", left: -3, bottom: Math.max(0, Math.min(50, 55 * volume() - 5)), width: 10, height: 10, borderRadius: 5, borderWidth: 1, borderColor: pageColor(), backgroundColor: accentColor() }} />
            </View>
          </View>
        ) : null}
        <View
          accessibilityLabel={t("Mute", "静音")}
          onPress={() => {
            const next = volume() > 0 ? 0 : 0.8;
            volume.set(next);
            Music.setVolume(next);
            void Storage.setItem("zeedle.volume", String(next));
          }}
          hoverStyle={{ backgroundColor: hoverColor() }}
          activeStyle={{ backgroundColor: pressedColor() }}
          style={{ width: 28, height: 30, alignItems: "center", justifyContent: "center" }}
        >
          <LucideIcon name="volume2" size={20} color={hovered() ? accentColor() : secondaryColor()} />
        </View>
      </View>
    );
  }

  function SideButton(props: { icon: LucideName; label: string; target: Page }) {
    const selected = () => page() === props.target;
    return (
      <View
        testID={`nav-${props.target}`}
        accessibilityLabel={props.label}
        onPress={() => navigateToPage(props.target)}
        hoverStyle={{ backgroundColor: hoverColor() }}
        activeStyle={{ opacity: 0.72 }}
        style={{
          height: 38,
          position: "relative",
          marginBottom: 5,
          paddingLeft: 14,
          paddingRight: 10,
          gap: expanded() ? 10 : 0,
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "flex-start",
          borderRadius: 8,
          backgroundColor: selected() ? hoverColor() : "transparent",
        }}
      >
        {selected() ? <View pointerEvents="none" style={{ position: "absolute", left: 3, top: 11, width: 3, height: 16, borderRadius: 2, backgroundColor: accentColor() }} /> : null}
        <View style={{ width: 20, height: 18, alignItems: "center", justifyContent: "center" }}>
          <LucideIcon name={props.icon} size={18} color={selected() ? accentColor() : secondaryColor()} />
        </View>
        {expanded() ? <Text style={textStyle(13, selected() ? textColor() : secondaryColor(), selected() ? "600" : "400")}>{props.label}</Text> : null}
      </View>
    );
  }

  function SongRows(props: { items: SongInfo[] }) {
    return (
      <List
        each={props.items}
        keyExtractor={(song) => song.song_path}
        estimatedItemSize={30}
        style={{ flex: 1, backgroundColor: pageColor() }}
        contentStyle={{ paddingBottom: 0 }}
        empty={
          <View style={{ flex: 1, minHeight: 220, alignItems: "center", justifyContent: "center", gap: 8 }}>
            <LucideIcon name="search" size={24} color={mutedColor()} />
            <Text style={textStyle(13, mutedColor())}>{loading() ? t("Reading your music library…", "正在读取音乐库…") : t("No songs found in this folder", "此文件夹中没有歌曲")}</Text>
            <Text style={textStyle(12, mutedColor())}>{t("Choose a folder with MP3, FLAC, WAV or OGG files", "选择包含 MP3、FLAC、WAV 或 OGG 文件的文件夹")}</Text>
          </View>
        }
      >
        {(item) => {
          const isPlaying = () => current()?.song_path === item().song_path;
          const hovered = signal(false);
          const openActions = () => {
            const selected = item();
            void Menu.popup([
              { label: t("Edit metadata…", "编辑元数据…"), onSelect: () => void beginEdit(selected) },
              { type: "separator" },
              { label: t("Delete song…", "删除歌曲…"), onSelect: () => deleteTarget.set(selected) },
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
              onHoverIn={() => hovered.set(true)}
              onHoverOut={() => hovered.set(false)}
              activeStyle={{ backgroundColor: pressedColor() }}
              style={{
                height: 30,
                flexDirection: "row",
                alignItems: "center",
                justifyContent: "space-between",
                backgroundColor: isPlaying() ? hoverColor() : hovered() ? songHoverColor() : "transparent",
              }}
            >
              <View style={{ width: "33%", justifyContent: "center" }}>
                <Text numberOfLines={1} style={{ ...textStyle(14, isPlaying() ? accentColor() : textColor(), isPlaying() ? "600" : "400"), width: "100%", marginLeft: "40%" }}>{item().song_name}</Text>
              </View>
              <View style={{ width: "33%", justifyContent: "center" }}>
                <Text numberOfLines={1} style={{ ...textStyle(14, isPlaying() ? accentColor() : textColor()), width: "100%", marginLeft: "40%" }}>{item().singer}</Text>
              </View>
              <View style={{ width: "33%", justifyContent: "center" }}>
                <Text style={{ ...textStyle(14, isPlaying() ? accentColor() : textColor()), marginLeft: "40%" }}>{item().duration}</Text>
              </View>
            </View>
          );
        }}
      </List>
    );
  }

  function GalleryPage() {
    return (
      <View testID="screen-0" style={{ flex: 1, minHeight: 0, backgroundColor: pageColor() }}>
        <View style={{ height: 30, flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
          <View onPress={() => { if (sortField() === "song_name") ascending.set(!ascending()); else { sortField.set("song_name"); ascending.set(true); } }} hoverStyle={{ backgroundColor: hoverColor() }} style={{ width: "33%", flexDirection: "row", alignItems: "center", gap: 5 }}>
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>{t("Title", "标题")}</Text>
            {sortField() === "song_name" ? <LucideIcon name={ascending() ? "arrowUp" : "arrowDown"} size={12} color={mutedColor()} /> : null}
          </View>
          <View onPress={() => { if (sortField() === "singer") ascending.set(!ascending()); else { sortField.set("singer"); ascending.set(true); } }} hoverStyle={{ backgroundColor: hoverColor() }} style={{ width: "33%", flexDirection: "row", alignItems: "center", gap: 5 }}>
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>{t("Artist", "艺术家")}</Text>
            {sortField() === "singer" ? <LucideIcon name={ascending() ? "arrowUp" : "arrowDown"} size={12} color={mutedColor()} /> : null}
          </View>
          <View onPress={() => { if (sortField() === "duration_secs") ascending.set(!ascending()); else { sortField.set("duration_secs"); ascending.set(true); } }} hoverStyle={{ backgroundColor: hoverColor() }} style={{ width: "33%", flexDirection: "row", alignItems: "center", gap: 5 }}>
            <Text style={{ ...textStyle(14, mutedColor()), marginLeft: "40%" }}>{t("Duration", "时长")}</Text>
            {sortField() === "duration_secs" ? <LucideIcon name={ascending() ? "arrowUp" : "arrowDown"} size={12} color={mutedColor()} /> : null}
          </View>
        </View>
        <SongRows items={displayedSongs()} />
      </View>
    );
  }

  function SearchPage() {
    return (
      <View testID="screen-1" style={{ flex: 1, minHeight: 0, backgroundColor: pageColor() }}>
        <View style={{ height: 40, paddingHorizontal: 12, flexDirection: "row", alignItems: "center", backgroundColor: pageColor() }}>
          <View style={{ flex: 1, height: 24, marginLeft: 3, marginRight: 2, flexDirection: "row", alignItems: "center", borderRadius: 3, borderBottomWidth: 2, borderBottomColor: accentColor() }}>
            <Input
              testID="song-search"
              value={query()}
              placeholder={t("Search songs or artists...", "搜索歌曲或艺术家…")}
              onChange={updateSearch}
              style={{ flex: 1, height: 24, paddingHorizontal: 4, color: textColor(), fontSize: 13, backgroundColor: "transparent" }}
            />
            {query() ? <View onPress={() => updateSearch("")} style={{ marginRight: 6, padding: 3 }}><LucideIcon name="x" size={14} color={mutedColor()} /></View> : null}
          </View>
        </View>
        {search() ? (
          <View style={{ height: 26, justifyContent: "center", paddingHorizontal: 12 }}>
            <Text style={textStyle(12, mutedColor())}>{language() === "zh" ? `${searchResults().length} ${t("result(s) found", "个结果")}` : `${searchResults().length} ${t("result(s) found", "result(s) found")}`}</Text>
          </View>
        ) : null}
        {search() ? <SongRows items={searchResults()} /> : (
          <View style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 8 }}>
            <LucideIcon name="search" size={44} color={mutedColor()} />
            <Text style={textStyle(14, mutedColor())}>{t("Type to search your music", "输入关键词开始搜索")}</Text>
          </View>
        )}
      </View>
    );
  }

  function LyricsPage() {
    const panelHeight = signal(470);
    const albumSize = () => Math.max(120, panelHeight() * 0.4);
    const initialLyricIndex = () => Math.max(0, lyricIndexAt(position()) - 5);
    return (
      <View testID="screen-2" onLayout={(event) => panelHeight.set(event.size.height)} style={{ flex: 1, minHeight: 0, flexDirection: "row", backgroundColor: pageColor() }}>
        <View style={{ width: lyrics().length ? "30%" : "100%", padding: 20, alignItems: "center", justifyContent: "center" }}>
          <View style={{ width: albumSize(), height: albumSize(), maxWidth: "100%", borderRadius: 9, overflow: "hidden", backgroundColor: raisedColor(), boxShadow: `0 12px 28px ${shadowColor()}` }}>
            <Image src={art()} style={{ width: "100%", height: "100%" }} />
          </View>
          <View style={{ width: albumSize(), marginTop: 5, gap: 3 }}>
            <Text numberOfLines={1} style={{ ...textStyle(14, textColor()), textAlign: lyrics().length ? "left" : "center" }}>{t("Artist:", "艺术家：")} {current()?.singer ?? t("Unknown", "未知")}</Text>
            <Text numberOfLines={1} style={{ ...textStyle(14, textColor()), textAlign: lyrics().length ? "left" : "center" }}>{t("Title:", "歌曲：")} {current()?.song_name ?? t("No song", "无歌曲")}</Text>
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
          style={{ width: "70%", flex: 1 }}
          contentStyle={{ paddingTop: 4, paddingBottom: 12 }}
        >
          {(line, index) => {
              const active = () => position() >= line().time && position() < line().time + line().duration;
              return (
                <View testID={active() ? "lyric-active" : `lyric-${index()}`} onPress={() => seekTo(line().time)} style={{ height: 40, alignItems: "center", justifyContent: "center" }}>
                  <Text numberOfLines={1} style={textStyle(active() ? 20 : 17, active() ? accentColor() : textColor(), active() ? "600" : "400")}>{line().text}</Text>
                </View>
              );
            }}
          </List>
        ) : null}
      </View>
    );
  }

  function SettingsPage() {
    const cardStyle = () => ({
      padding: 16,
      borderRadius: 8,
      borderWidth: 1,
      borderColor: lineColor(),
      backgroundColor: panelColor(),
      gap: 10,
    });
    const changeFollowSystem = () => {
      const next = !followSystem();
      if (next) {
        followSystem.set(true);
        setColorScheme("system");
        void Storage.removeItem("zeedle.light-theme");
      } else {
        const currentScheme = colorScheme();
        followSystem.set(false);
        setColorScheme(currentScheme);
        void Storage.setItem("zeedle.light-theme", String(currentScheme === "light"));
      }
    };
    const changeSpectrum = () => {
      const next = !showSpectrum();
      showSpectrum.set(next);
      Music.setSpectrumEnabled(next);
      void Storage.setItem("zeedle.show-spectrum", String(next));
    };
    const checkbox = (checked: boolean, onPress: () => void) => (
      <View
        onPress={onPress}
        style={{
          width: 18,
          height: 18,
          borderRadius: 2,
          borderWidth: checked ? 0 : 1,
          borderColor: checkboxBorderColor(),
          backgroundColor: checked ? accentColor() : "transparent",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {checked ? <LucideIcon name="check" size={15} color="#ffffff" /> : null}
      </View>
    );

    return (
      <View testID="screen-3" style={{ flex: 1, minHeight: 0, backgroundColor: pageColor() }}>
      <Scroll testID="settings-scroll" style={{ flex: 1, backgroundColor: pageColor() }} contentStyle={{ padding: 24, gap: 20 }}>
        <View style={cardStyle()}>
          <Text style={textStyle(12, mutedColor(), "600")}>{t("Playback", "播放")}</Text>
          <View style={{ height: 1, backgroundColor: dividerColor() }} />
          <View style={{ gap: 6 }}>
            <Text style={textStyle(13, secondaryColor())}>{t("Music directory", "音乐文件夹")}</Text>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
              <View style={{ flex: 1, minWidth: 0, height: 32, justifyContent: "center", paddingHorizontal: 12, borderRadius: 4, borderWidth: 1, borderColor: inputBorderColor(), backgroundColor: raisedColor() }}>
                <Text numberOfLines={1} style={textStyle(13, secondaryColor())}>{directory()}</Text>
              </View>
              <View onPress={() => void chooseDirectory()} style={{ paddingHorizontal: 12, height: 32, flexDirection: "row", alignItems: "center", justifyContent: "center", borderRadius: 4, backgroundColor: buttonColor() }}>
                <Text style={textStyle(13, textColor(), "500")}>{t("Browse…", "浏览…")}</Text>
              </View>
            </View>
          </View>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
            {checkbox(showSpectrum(), changeSpectrum)}
            <Text onPress={changeSpectrum} style={textStyle(13, textColor())}>{t("Spectrum display", "频谱显示")}</Text>
          </View>
        </View>
        <View style={cardStyle()}>
          <Text style={textStyle(12, mutedColor(), "600")}>{t("Appearance", "外观")}</Text>
          <View style={{ height: 1, backgroundColor: dividerColor() }} />
          <View style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
            {checkbox(followSystem(), changeFollowSystem)}
            <Text onPress={changeFollowSystem} style={textStyle(13, textColor())}>{t("Follow system theme", "跟随系统主题")}</Text>
          </View>
          <View style={{ gap: 6 }}>
            <Text style={textStyle(13, secondaryColor())}>{t("Language", "语言")}</Text>
            <View style={{ flexDirection: "row", gap: 8 }}>
              {[
                { code: "en", label: "English" },
                { code: "zh", label: "中文" },
                { code: "es", label: "Español" },
                { code: "fr", label: "Français" },
                { code: "de", label: "Deutsch" },
                { code: "ru", label: "Русский" },
              ].map((option) => (
                <View
                  testID={`language-${option.code}`}
                  onPress={() => {
                    language.set(option.code as Language);
                    void Storage.setItem("zeedle.language", option.code);
                  }}
                  style={{
                    flex: 1,
                    minWidth: 0,
                    height: 26,
                    alignItems: "center",
                    justifyContent: "center",
                    borderRadius: 4,
                    borderWidth: 1,
                    borderColor: language() === option.code ? accentColor() : inputBorderColor(),
                    backgroundColor: language() === option.code ? accentColor() : hoverColor(),
                  }}
                >
                  <Text numberOfLines={1} style={textStyle(13, language() === option.code ? primaryForegroundColor() : textColor())}>{option.label}</Text>
                </View>
              ))}
            </View>
          </View>
        </View>
      </Scroll>
      </View>
    );
  }

  function AboutPage() {
    return (
      <View testID="screen-4" style={{ flex: 1, alignItems: "center", justifyContent: "center", backgroundColor: pageColor(), gap: 10 }}>
        <Image src={cover} style={{ width: 120, height: 120 }} />
        <Text numberOfLines={2} style={{ ...textStyle(14, textColor()), textAlign: "center" }}>
          {`Zeedle\n${t("Music at the speed of a blink, designed for pure simplicity", "以眨眼般的速度享受音乐，追求纯粹简洁")}`}
        </Text>
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "center", gap: 4 }}>
          <Text style={textStyle(14, textColor())}>{t("Built by", "构建者")}</Text>
          <View role="link" accessibilityLabel="Rodio" onPress={() => void openExternalUrl("https://github.com/RustAudio/rodio")}>
            <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>Rodio</Text>
          </View>
          <Text style={textStyle(14, textColor())}>{t("and", "和")}</Text>
          <View testID="about-nsh-link" role="link" accessibilityLabel="NSH" onPress={() => void openExternalUrl("https://nsh.dev")}>
            <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>NSH</Text>
          </View>
        </View>
        <Text numberOfLines={2} style={{ ...textStyle(14, textColor()), textAlign: "center" }}>{t("Author:", "作者：")} Jordan Haidee{"\n"}{t("Version:", "版本：")} 0.1.1</Text>
        <View role="link" accessibilityLabel="Github" onPress={() => void openExternalUrl("https://github.com/Jordan-Haidee/Zeedle")}>
          <Text style={{ ...textStyle(14, accentColor()), textDecorationLine: "underline" }}>Github</Text>
        </View>
      </View>
    );
  }

  function PlayerPanel() {
    const progressPercent = () => duration() > 0 ? Math.max(0, Math.min(100, (position() / duration()) * 100)) : 0;
    return (
      <View style={{ height: 72, minHeight: 72, backgroundColor: pageColor() }}>
        <View
          testID="progress-seek"
          accessibilityLabel={t("Playback progress", "播放进度")}
          accessibilityValue={{ min: 0, max: duration(), now: position() }}
          role="adjustable"
          hitSlop={{ top: 8, bottom: 8 }}
          onPress={(event) => seekTo(progressAtX(event?.position.x ?? 0))}
          onLayout={(event) => trackWidth.set(event.size.width)}
          style={{ height: 4, marginHorizontal: 10, backgroundColor: trackColor(), justifyContent: "center" }}
        >
          <View style={{ height: 3, width: `${progressPercent()}%`, backgroundColor: accentColor() }} />
          <View style={{ position: "absolute", left: `${progressPercent()}%`, top: -8, marginLeft: -10, width: 20, height: 20, borderRadius: 10, borderWidth: 3, borderColor: trackThumbBorderColor(), backgroundColor: accentColor() }} />
        </View>
        <View style={{ height: 68, flexDirection: "row", alignItems: "center", paddingHorizontal: 0 }}>
          <View style={{ width: "35%", flexDirection: "row", alignItems: "center", gap: 3 }} onPress={() => navigateToPage(2)}>
            <View style={{ width: "20%", alignItems: "center", justifyContent: "center" }}>
              <View style={{ width: 40, height: 40, borderRadius: 4, overflow: "hidden" }}>
                <Image src={art()} style={{ width: 40, height: 40 }} />
              </View>
            </View>
            <View style={{ width: "80%" }}>
              <Text numberOfLines={1} style={{ ...textStyle(14, textColor()), textAlign: "center" }}>
                {current() ? `${current()!.song_name} - ${current()!.singer}` : t("No song selected", "尚未选择歌曲")}
              </Text>
            </View>
          </View>
          <View style={{ width: "6%", flexDirection: "row", alignItems: "center", justifyContent: "center" }}>
            <View testID="control-repeat" accessibilityLabel={t("Repeat current track", "单曲循环")} accessibilityState={{ selected: mode() !== "random" }} onPress={() => changeMode(mode() === "order" ? "repeat" : "order")} style={controlStyle(20)}>
              <LucideIcon name={mode() === "repeat" ? "repeat2" : "repeat"} size={18} color={mode() !== "random" ? accentColor() : secondaryColor()} />
            </View>
          </View>
          <View style={{ width: "6%", flexDirection: "row", alignItems: "center", justifyContent: "center" }}>
            <PlayerTransportButton testID="control-previous" label={t("Previous track", "上一首")} name="skipBack" color={secondaryColor()} onPress={playPrevious} radius={8} />
          </View>
          <View style={{ width: "6%", flexDirection: "row", alignItems: "center", justifyContent: "center" }}>
            <PlayerTransportButton testID="control-play-pause" label={paused() ? t("Play", "播放") : t("Pause", "暂停")} name={paused() ? "play" : "pause"} color={accentColor()} onPress={togglePlay} />
          </View>
          <View style={{ width: "6%", flexDirection: "row", alignItems: "center", justifyContent: "center" }}>
            <PlayerTransportButton testID="control-next" label={t("Next track", "下一首")} name="skipForward" color={secondaryColor()} onPress={playNext} radius={8} />
          </View>
          <View style={{ width: "6%", flexDirection: "row", alignItems: "center", justifyContent: "center" }}>
            <View testID="control-shuffle" accessibilityLabel={t("Shuffle", "随机播放")} accessibilityState={{ selected: mode() === "random" }} onPress={() => changeMode("random")} style={controlStyle(20)}>
              <LucideIcon name="shuffle" size={18} color={mode() === "random" ? accentColor() : secondaryColor()} />
            </View>
          </View>
          <View style={{ width: "25%", alignItems: "center", justifyContent: "center" }}>
            <Text style={textStyle(14, textColor())}>{timeLabel(position())} / {timeLabel(duration())}</Text>
          </View>
          <View style={{ width: "10%", alignItems: "center", justifyContent: "center" }}>
            <VolumeControl />
          </View>
        </View>
      </View>
    );
  }

  return (
    <View style={{ flex: 1, width: 900, height: 510, minWidth: 850, minHeight: 500, flexDirection: "column", backgroundColor: pageColor(), borderWidth: 1, borderColor: lineColor(), overflow: "hidden" }}>
      <View
        desktop={{ windowDrag: true }}
        style={{ height: titleBar().height || 40, minHeight: 40, flexDirection: "row", alignItems: "center", paddingLeft: titleBar().left + 10, paddingRight: titleBar().right, backgroundColor: panelColor(), borderBottomWidth: 1, borderBottomColor: lineColor() }}
      >
        <View style={{ width: 18, height: 18, borderRadius: 4, overflow: "hidden", marginRight: 6 }}>
          <Image src={cover} style={{ width: 18, height: 18 }} />
        </View>
        <Text style={textStyle(13, textColor(), "600")}>Zeedle</Text>
        <View style={{ flex: 1 }} />
          <View
            testID="theme-toggle"
            accessibilityLabel={t("Toggle theme", "切换主题")}
            disabled={followSystem()}
            disabledStyle={{ opacity: 0.4 }}
            hoverStyle={{ backgroundColor: hoverColor() }}
            onPress={() => {
              if (followSystem()) return;
              const next = !light();
              setColorScheme(next ? "light" : "dark");
              void Storage.setItem("zeedle.light-theme", String(next));
            }}
            style={{ width: 42, height: 40, alignItems: "center", justifyContent: "center" }}
          >
          <LucideIcon name={light() ? "sun" : "moon"} size={18} color={secondaryColor()} />
        </View>
        {titleBar().right === 0 ? (
          <View style={{ flexDirection: "row", alignItems: "center" }}>
            <View style={{ width: 1, height: 16, backgroundColor: dividerColor() }} />
            <View onPress={() => Window.minimize()} hoverStyle={{ backgroundColor: hoverColor() }} style={{ width: 42, height: 40, alignItems: "center", justifyContent: "center" }}>
            <LucideIcon name="minus" size={18} color={secondaryColor()} />
            </View>
            <View onPress={() => Window.toggleMaximize()} hoverStyle={{ backgroundColor: hoverColor() }} style={{ width: 42, height: 40, alignItems: "center", justifyContent: "center" }}>
            <LucideIcon name="square" size={12} color={secondaryColor()} />
            </View>
            <View onPress={() => Window.close()} hoverStyle={{ backgroundColor: "#e81123" }} style={{ width: 42, height: 40, alignItems: "center", justifyContent: "center" }}>
              <LucideIcon name="x" size={14} color={secondaryColor()} />
            </View>
          </View>
        ) : null}
      </View>

      <View style={{ flex: 1, minHeight: 0, flexDirection: "row" }}>
        <View style={{ width: expanded() ? 200 : 54, backgroundColor: panelColor(), borderRightWidth: 1, borderRightColor: lineColor(), paddingHorizontal: 6, paddingTop: 9, paddingBottom: 10 }}>
          <View testID="sidebar-toggle" onPress={() => {
            LayoutAnimation.configureNext({
              duration: 200,
              update: { type: "easeOut" },
              create: { type: "easeOut", property: "opacity", duration: 120 },
              delete: { type: "easeOut", property: "opacity", duration: 120 },
            });
            expanded.set(!expanded());
          }} hoverStyle={{ backgroundColor: hoverColor() }} style={{ height: 36, marginBottom: 10, paddingLeft: 14, paddingRight: 10, flexDirection: "row", alignItems: "center", justifyContent: "flex-start", gap: 10, borderRadius: 8 }}>
            <View style={{ width: 20, height: 18, alignItems: "center", justifyContent: "center" }}>
              <LucideIcon name={expanded() ? "panelLeftClose" : "panelLeftOpen"} size={18} color={secondaryColor()} />
            </View>
          </View>
          <View style={{ height: 5 }} />
          <SideButton icon="listMusic" label={t("Gallery", "曲库")} target={0} />
          <SideButton icon="search" label={t("Search", "搜索")} target={1} />
          <SideButton icon="captions" label={t("Lyrics", "歌词")} target={2} />
          <View style={{ flex: 1 }} />
          <View style={{ height: 1, backgroundColor: lineColor(), marginHorizontal: 1, marginBottom: 8 }} />
          <SideButton icon="settings" label={t("Settings", "设置")} target={3} />
          <SideButton icon="circleHelp" label={t("About", "关于")} target={4} />
        </View>

        <View style={{ flex: 1, minWidth: 0, minHeight: 0, position: "relative", flexDirection: "column", backgroundColor: pageColor() }}>
          <View style={{ flex: 1, minHeight: 0, backgroundColor: pageColor() }}>
            {page() === 0 ? <GalleryPage /> : null}
            {page() === 1 ? <SearchPage /> : null}
            {page() === 2 ? <LyricsPage /> : null}
            {page() === 3 ? <SettingsPage /> : null}
            {page() === 4 ? <AboutPage /> : null}
          </View>
          {page() === 0 ? <PlayerPanel /> : null}
          {showSpectrum() ? (
            <View pointerEvents="none" style={{ position: "absolute", left: 0, right: 0, bottom: 0, height: 60, alignItems: "center", justifyContent: "flex-end", zIndex: 5 }}>
              <View style={{ width: "50%", height: 60, flexDirection: "row", alignItems: "flex-end", opacity: 0.3 }}>
                {spectrumBars().map((bar) => <View style={{ flex: 1, height: Math.max(0, Math.min(1, bar)) * 60, borderRadius: 1, backgroundColor: accentColor() }} />)}
              </View>
            </View>
          ) : null}
        </View>
      </View>

      {playerError() ? (
        <View onPress={() => playerError.set("")} style={{ position: "absolute", right: 15, bottom: page() === 0 ? 82 : 12, width: "80%", maxWidth: 334, paddingHorizontal: 12, paddingVertical: 9, flexDirection: "row", alignItems: "center", gap: 8, backgroundColor: dangerSurfaceColor(), borderRadius: 7, borderWidth: 1, borderColor: dangerColor() }}>
          <LucideIcon name="circleAlert" size={16} color={dangerColor()} />
          <Text numberOfLines={2} style={{ ...textStyle(11, dangerTextColor()), flex: 1, minWidth: 0 }}>{playerError()}</Text>
        </View>
      ) : null}

      {editingSong() ? (
        <View style={{ position: "absolute", top: 0, right: 0, bottom: 0, left: 0, zIndex: 20, alignItems: "center", justifyContent: "center", padding: 10 }}>
          <View style={{ width: 520, height: 450, maxWidth: "96%", maxHeight: "96%", paddingHorizontal: 16, paddingTop: 14, paddingBottom: 16, borderRadius: 8, borderWidth: 1, borderColor: lineColor(), backgroundColor: pageColor(), gap: 7.5 }}>
            <Text style={textStyle(18, textColor(), "600")}>{t("Edit song metadata", "编辑歌曲元信息")}</Text>
            <View style={{ gap: 4 }}>
              <Text style={textStyle(14, secondaryColor())}>{t("Song title", "歌曲名")}</Text>
              <Input testID="edit-title" value={editTitle()} onChange={editTitle.set} style={{ height: 32, paddingHorizontal: 10, borderWidth: 1, borderColor: lineColor(), borderRadius: 4, color: textColor(), backgroundColor: raisedColor(), fontSize: 14 }} />
            </View>
            <View style={{ flexDirection: "row", gap: 12 }}>
              <View style={{ flex: 1, gap: 4 }}>
                <Text style={textStyle(14, secondaryColor())}>{t("Artist", "艺术家")}</Text>
                <Input testID="edit-artist" value={editArtist()} onChange={editArtist.set} style={{ height: 32, paddingHorizontal: 10, borderWidth: 1, borderColor: lineColor(), borderRadius: 4, color: textColor(), backgroundColor: raisedColor(), fontSize: 14 }} />
              </View>
              <View style={{ flex: 1, gap: 4 }}>
                <Text style={textStyle(14, secondaryColor())}>{t("Album", "专辑")}</Text>
                <Input testID="edit-album" value={editAlbum()} onChange={editAlbum.set} style={{ height: 32, paddingHorizontal: 10, borderWidth: 1, borderColor: lineColor(), borderRadius: 4, color: textColor(), backgroundColor: raisedColor(), fontSize: 14 }} />
              </View>
            </View>
            <View style={{ height: 200, minHeight: 200, marginTop: -1, flexDirection: "row", gap: 10 }}>
              <View style={{ flex: 1, minWidth: 0, paddingLeft: 8, paddingRight: 10, paddingTop: 8, paddingBottom: 10, gap: 6, borderWidth: 1, borderColor: lineColor(), borderRadius: 8, backgroundColor: panelColor() }}>
                <Text style={textStyle(14, textColor(), "600")}>{t("Album cover", "专辑封面")}</Text>
                <View style={{ marginTop: -1, flexDirection: "row", alignItems: "flex-start", minHeight: 0 }}>
                  <Image src={editCoverPreview()} style={{ width: 100, height: 100 }} />
                </View>
                <View onPress={() => void chooseEditCover()} style={{ height: 32, alignItems: "center", justifyContent: "center", borderRadius: 4, backgroundColor: raisedColor() }}>
                  <Text style={textStyle(13, textColor())}>{t("Select cover image", "选择封面图片")}</Text>
                </View>
                <Text style={textStyle(11, mutedColor())}>{editCoverPath() ? editCoverName() : t("Existing cover will be kept", "将保留现有封面")}</Text>
              </View>
              <View style={{ flex: 1, minWidth: 0, paddingHorizontal: 10, paddingTop: 8, paddingBottom: 10, gap: 6, borderWidth: 1, borderColor: lineColor(), borderRadius: 8, backgroundColor: panelColor() }}>
                <Text style={textStyle(14, textColor(), "600")}>{t("Lyrics", "歌词")}</Text>
                <Scroll style={{ height: 100, minHeight: 100, marginTop: -1, padding: 8, borderRadius: 4, backgroundColor: raisedColor() }}>
                  {(editLyricsPreview() || t("No embedded lyrics", "没有内嵌歌词")).split(/\r?\n/).map((line) => (
                  <Text numberOfLines={1} style={{ ...textStyle(11, textColor()), lineHeight: 13 }}>{line}</Text>
                  ))}
                </Scroll>
                <View onPress={() => void chooseEditLyrics()} style={{ height: 32, alignItems: "center", justifyContent: "center", borderRadius: 4, backgroundColor: raisedColor() }}>
                  <Text style={textStyle(13, textColor())}>{t("Select LRC file", "选择 LRC 歌词文件")}</Text>
                </View>
                <Text style={textStyle(11, mutedColor())}>{editLyricsPath() ? editLyricsName() : t("Existing lyrics will be kept", "将保留现有歌词")}</Text>
              </View>
            </View>
            {editError() ? <Text style={textStyle(11, dangerColor())}>{editError()}</Text> : null}
            <View style={{ flex: 1 }} />
            <View style={{ height: 1, backgroundColor: lineColor() }} />
            <View style={{ flexDirection: "row", justifyContent: "flex-end", gap: 8 }}>
              <View testID="edit-cancel" onPress={() => editingSong.set(null)} style={{ width: 68, height: 32, alignItems: "center", justifyContent: "center", borderRadius: 4, backgroundColor: raisedColor() }}>
                <Text style={textStyle(13, textColor())}>{t("Cancel", "取消")}</Text>
              </View>
              <View onPress={() => void saveEdit()} style={{ width: 55, height: 32, alignItems: "center", justifyContent: "center", borderRadius: 4, backgroundColor: buttonColor(), opacity: editSaving() ? 0.65 : 1 }}>
                <Text style={textStyle(13, textColor(), "500")}>{editSaving() ? t("Saving…", "正在保存…") : t("Save", "保存")}</Text>
              </View>
            </View>
          </View>
        </View>
      ) : null}

      {deleteTarget() ? (
        <View style={{ position: "absolute", top: 0, right: 0, bottom: 0, left: 0, zIndex: 21, alignItems: "center", justifyContent: "center", padding: 20 }}>
          <View style={{ width: 370, maxWidth: "100%", padding: 20, borderRadius: 8, borderWidth: 1, borderColor: lineColor(), backgroundColor: pageColor(), gap: 11 }}>
            <Text style={textStyle(17, textColor(), "600")}>{t("Delete this song?", "删除这首歌曲？")}</Text>
            <Text style={textStyle(13, textColor(), "500")}>{deleteTarget()?.song_name}</Text>
            <Text numberOfLines={2} style={textStyle(11, secondaryColor())}>{t("The audio file will be removed from your device.", "此音频文件将从设备中删除。")}</Text>
            <View style={{ flexDirection: "row", justifyContent: "flex-end", gap: 8, marginTop: 5 }}>
              <View testID="delete-cancel" onPress={() => deleteTarget.set(null)} style={{ paddingHorizontal: 13, paddingVertical: 8, borderRadius: 4, backgroundColor: raisedColor() }}>
                <Text style={textStyle(12, secondaryColor(), "500")}>{t("Cancel", "取消")}</Text>
              </View>
              <View onPress={() => void confirmDelete()} style={{ paddingHorizontal: 14, paddingVertical: 8, borderRadius: 4, backgroundColor: dangerColor() }}>
                <Text style={textStyle(12, token("destructive-foreground"), "600")}>{t("Delete song", "删除歌曲")}</Text>
              </View>
            </View>
          </View>
        </View>
      ) : null}
    </View>
  );
}

mount(App);
