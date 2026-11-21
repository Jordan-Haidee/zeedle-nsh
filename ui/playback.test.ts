import { expect, test } from "nsh/test";
import { selectNextSong, selectPreviousSong } from "./playback";
import type { SongInfo } from "./types";

const songs = ["a", "b", "c"].map((song_path, index) => ({
  id: index,
  song_name: song_path,
  singer: "artist",
  duration: "1:00",
  duration_secs: 60,
  song_path,
})) satisfies SongInfo[];

test("ordered navigation follows playlist order and wraps at either end", () => {
  expect(selectNextSong(songs, "c", "order", [], -1)?.song_path).toBe("a");
  expect(selectPreviousSong(songs, "a", "order", [], -1)?.song_path).toBe("c");
  expect(selectPreviousSong(songs, null, "order", [], -1)?.song_path).toBe("c");
  expect(selectNextSong(songs, null, "order", [], -1)?.song_path).toBe("a");
});

test("repeat next keeps the current song while previous remains navigable", () => {
  expect(selectNextSong(songs, "b", "repeat", ["a", "b", "c"], 0)?.song_path).toBe("b");
  expect(selectPreviousSong(songs, "b", "repeat", [], -1)?.song_path).toBe("a");
});

test("random navigation avoids the current song and follows visited history", () => {
  expect(selectNextSong(songs, "b", "random", [], -1, 0)?.song_path).toBe("a");
  expect(selectNextSong(songs, "b", "random", ["a", "b", "c"], 1)?.song_path).toBe("c");
  expect(selectPreviousSong(songs, "c", "random", ["a", "b", "c"], 2)?.song_path).toBe("b");
  expect(selectPreviousSong(songs, "c", "random", [], -1)?.song_path).toBe("b");
});

test("navigation safely handles an empty library and a one-song shuffle", () => {
  expect(selectNextSong([], null, "order", [], -1)).toBe(null);
  expect(selectPreviousSong([], null, "random", [], -1)).toBe(null);
  expect(selectNextSong(songs.slice(0, 1), "a", "random", [], -1)?.song_path).toBe("a");
});
