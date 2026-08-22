use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMode {
    Shuffle,
    RepeatOne,
    RepeatList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub state: PlayState,
    pub mode: PlayMode,
    pub current_track_id: Option<i64>,
    pub position: f64,
    pub duration: f64,
    pub volume: f32,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            state: PlayState::Stopped,
            mode: PlayMode::RepeatList,
            current_track_id: None,
            position: 0.0,
            duration: 0.0,
            volume: 0.8,
        }
    }
}
