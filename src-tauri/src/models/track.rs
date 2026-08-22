use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub duration: i64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub cover_blob: Option<String>,
    pub lyrics: Option<String>,
    pub added_at: i64,
    pub modified_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackFilter {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
