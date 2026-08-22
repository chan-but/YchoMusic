pub const CREATE_TRACKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    title TEXT,
    artist TEXT,
    album TEXT,
    year INTEGER,
    track_number INTEGER,
    duration INTEGER,
    bitrate INTEGER,
    sample_rate INTEGER,
    cover_blob BLOB,
    lyrics TEXT,
    scan_folder_id INTEGER,
    added_at INTEGER,
    modified_at INTEGER,
    FOREIGN KEY(scan_folder_id) REFERENCES scan_folders(id)
);
"#;

pub const CREATE_SCAN_FOLDERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS scan_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    added_at INTEGER
);
"#;

pub const CREATE_PLAYLISTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    sort_key TEXT DEFAULT 'title',
    sort_order TEXT DEFAULT 'asc',
    created_at INTEGER
);
"#;

pub const CREATE_PLAYLIST_ITEMS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    playlist_id INTEGER,
    track_id INTEGER,
    position INTEGER,
    FOREIGN KEY(playlist_id) REFERENCES playlists(id),
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);
"#;

pub const CREATE_PLAY_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS play_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER,
    played_at INTEGER,
    duration_played INTEGER,
    completed BOOLEAN,
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);
"#;

pub const CREATE_FAVORITES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS favorites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id INTEGER UNIQUE,
    added_at INTEGER,
    FOREIGN KEY(track_id) REFERENCES tracks(id)
);
"#;

pub const CREATE_SETTINGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
"#;

pub const CREATE_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
CREATE INDEX IF NOT EXISTS idx_playlist_items_playlist ON playlist_items(playlist_id);
CREATE INDEX IF NOT EXISTS idx_play_history_track ON play_history(track_id);
CREATE INDEX IF NOT EXISTS idx_play_history_played ON play_history(played_at);
"#;
