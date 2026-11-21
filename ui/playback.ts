import type { PlayMode, SongInfo } from "./types";

export function selectNextSong(
  songs: SongInfo[],
  currentPath: string | null,
  mode: PlayMode,
  history: string[],
  historyIndex: number,
  randomValue = Math.random(),
): SongInfo | null {
  if (!songs.length) return null;
  const current = songs.find((song) => song.song_path === currentPath);
  if (mode === "repeat" && current) return current;

  if (mode === "random") {
    const nextHistorySong = songs.find((song) => song.song_path === history[historyIndex + 1]);
    if (nextHistorySong) return nextHistorySong;
    const candidates = songs.filter((song) => song.song_path !== currentPath);
    const choices = candidates.length ? candidates : songs;
    return choices[Math.min(choices.length - 1, Math.floor(randomValue * choices.length))];
  }

  const index = current ? songs.indexOf(current) : -1;
  return songs[(index + 1) % songs.length];
}

export function selectPreviousSong(
  songs: SongInfo[],
  currentPath: string | null,
  mode: PlayMode,
  history: string[],
  historyIndex: number,
): SongInfo | null {
  if (!songs.length) return null;
  if (mode === "random") {
    const previousHistorySong = songs.find((song) => song.song_path === history[historyIndex - 1]);
    if (previousHistorySong) return previousHistorySong;
  }

  const index = songs.findIndex((song) => song.song_path === currentPath);
  if (index < 0) return songs[songs.length - 1];
  return songs[(index - 1 + songs.length) % songs.length];
}
