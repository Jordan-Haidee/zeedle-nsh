export type Page = 0 | 1 | 2 | 3 | 4;
export type SortField = "song_name" | "singer" | "duration_secs";
export type PlayMode = "order" | "repeat" | "random";

export type SongInfo = {
  id: number;
  song_name: string;
  singer: string;
  duration: string;
  duration_secs: number;
  song_path: string;
};

export type LyricItem = { time: number; duration: number; text: string };
export type PlaybackState = {
  position: number;
  duration: number;
  paused: boolean;
  volume: number;
  error: string;
  spectrum: number[];
};
export type AppConfigValue = {
  song_dir: string;
  current_song_path: string | null;
  progress: number;
  play_mode: "InOrder" | "Recursive" | "Random";
  sort_key: "BySongName" | "BySinger" | "ByDuration";
  sort_ascending: boolean;
  lang: string;
  light_ui: boolean;
  follow_system_theme: boolean;
  volume: number;
  show_spectrum: boolean;
  current_page: number;
  sidebar_expanded: boolean;
  gallery_viewport_y: number;
  close_to_tray: boolean;
};
