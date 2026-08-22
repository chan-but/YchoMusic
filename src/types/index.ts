export interface Track {
  id: number;
  path: string;
  title: string | null;
  artist: string | null;
  album: string | null;
  year: number | null;
  track_number: number | null;
  duration: number;
  bitrate: number | null;
  sample_rate: number | null;
  cover_blob: string | null;
  lyrics: string | null;
  scan_folder_id: number | null;
  added_at: number;
  modified_at: number;
}

export interface TrackFilter {
  artist?: string;
  album?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

export interface ScanFolder {
  id: number;
  path: string;
  name: string;
  track_count: number;
  added_at: number;
}

export interface AlbumInfo {
  id: string;
  name: string;
  artist: string | null;
  track_count: number;
  cover_blob: string | null;
  year: number | null;
}

export interface ArtistInfo {
  id: string;
  name: string;
  track_count: number;
  cover_blob: string | null;
}

export enum PlayState {
  Stopped = 'Stopped',
  Playing = 'Playing',
  Paused = 'Paused',
}

export enum PlayMode {
  Shuffle = 'Shuffle',
  RepeatOne = 'RepeatOne',
  RepeatList = 'RepeatList',
}

export interface PlayerState {
  state: PlayState;
  mode: PlayMode;
  current_track_id: number | null;
  position: number;
  duration: number;
  volume: number;
}

export interface Playlist {
  id: number;
  name: string;
  sort_key: string;
  sort_order: string;
  created_at: number;
}

export interface LyricLine {
  time: number;
  text: string;
}

export interface PlayStats {
  total_tracks_played: number;
  total_duration_played: number;
  top_tracks: TopTrack[];
}

export interface TopTrack {
  track_id: number;
  title: string;
  artist: string | null;
  play_count: number;
}

export interface PlayHistoryEntry {
  track_id: number;
  played_at: number;
  duration_played: number;
  completed: boolean;
}

export interface Settings {
  theme: string;
  volume: number;
  play_mode: string;
  auto_play: boolean;
  remember_position: boolean;
  cache_enabled: boolean;
  cache_max_size: number;
  click_cover_plays: boolean;
  playlist_auto_scroll: boolean;
  playlist_scroll_mode: 'ltr' | 'rtl' | 'bounce';
  playlist_scroll_speed: number;
  show_spectrum: boolean;
  lyrics_mode: 'embedded' | 'file';
  lyrics_lines: number;
  desktop_lyrics: boolean;
  desktop_lyrics_lines: number;
  lyrics_folder_path: string;
  show_audio_quality: boolean;
  language: string;
  close_behavior: 'ask' | 'tray' | 'exit';
  mini_on_top: boolean;
  mini_single_line_lyrics: boolean;
  font_size: number;
  font_family: string;
  lyrics_font_size: number;      // default 16, range 12-32
  lyrics_line_height: number;    // default 2.2, range 1.5-4.0
  scroll_loop_marker: boolean;   // default true - show marker when scroll loops
  shortcut_prev: string;
  shortcut_next: string;
  shortcut_play_pause: string;
  shortcut_mode: string;
  shortcut_mini: string;
  shortcut_fullscreen: string;
}

export type ViewMode = 'text' | 'cover' | 'large_cover';
