use std::sync::{Arc, Condvar, Mutex, mpsc::{self, Receiver, Sender}};
use std::time::{Duration, Instant};

use rodio::{OutputStream, Sink};

use super::decoder::{convert_audio_buffer, AudioDecoder};
use crate::models::{PlayMode, PlayState, PlayerState, Track};

enum PlayerCommand {
    Play(Track),
    Pause,
    Resume,
    Stop,
    Seek(u64),
    SetVolume(f32),
    SetMode(PlayMode),
    SetPlaylist(Vec<Track>),
}

struct PlayerStateInternal {
    play_state: PlayState,
    play_mode: PlayMode,
    current_track: Option<Track>,
    playlist: Vec<Track>,
    position: f64,
    duration: f64,
    volume: f32,
    total_decoded_frames: u64,
    last_position_update: Instant,
}

impl Default for PlayerStateInternal {
    fn default() -> Self {
        Self {
            play_state: PlayState::Stopped,
            play_mode: PlayMode::RepeatList,
            current_track: None,
            playlist: Vec::new(),
            position: 0.0,
            duration: 0.0,
            volume: 0.8,
            total_decoded_frames: 0,
            last_position_update: Instant::now(),
        }
    }
}

type PlayAck = (Mutex<Option<Result<(), String>>>, Condvar);

pub struct PlayerHandle {
    sender: Mutex<Option<Sender<PlayerCommand>>>,
    state: Arc<Mutex<PlayerStateInternal>>,
    play_ack: Arc<PlayAck>,
}

impl PlayerHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(PlayerStateInternal::default()));
        let state_clone = state.clone();
        let play_ack: Arc<PlayAck> = Arc::new((Mutex::new(None), Condvar::new()));
        let play_ack_clone = play_ack.clone();

        std::thread::spawn(move || {
            audio_thread_main(receiver, state_clone, play_ack_clone);
        });

        PlayerHandle {
            sender: Mutex::new(Some(sender)),
            state,
            play_ack,
        }
    }

    fn send(&self, cmd: PlayerCommand) {
        if let Some(sender) = self.sender.lock().unwrap().as_ref() {
            let _ = sender.send(cmd);
        }
    }

    pub fn set_playlist(&self, tracks: Vec<Track>) {
        self.send(PlayerCommand::SetPlaylist(tracks));
    }

    pub fn play(&self, track: &Track) -> Result<(), String> {
        {
            let mut ack = self.play_ack.0.lock().unwrap();
            *ack = None;
        }

        self.send(PlayerCommand::Play(track.clone()));

        let result;
        {
            let ack = self.play_ack.0.lock().unwrap();
            let timeout = Duration::from_millis(500);
            match self.play_ack.1.wait_timeout(ack, timeout) {
                Ok((mut guard, _)) => {
                    result = guard.take();
                }
                Err(_timeout) => {
                    return Ok(());
                }
            }
        }

        match result {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(e),
            None => Ok(()),
        }
    }

    pub fn pause(&self) {
        self.send(PlayerCommand::Pause);
    }

    pub fn resume(&self) {
        self.send(PlayerCommand::Resume);
    }

    pub fn stop(&self) {
        self.send(PlayerCommand::Stop);
    }

    pub fn prev(&self) {
        let state = self.state.lock().unwrap();
        if state.playlist.is_empty() {
            return;
        }
        let current_idx = state
            .playlist
            .iter()
            .position(|t| state.current_track.as_ref().map(|c| c.id == t.id).unwrap_or(false))
            .unwrap_or(0);

        let new_idx = if current_idx == 0 {
            state.playlist.len() - 1
        } else {
            current_idx - 1
        };

        if let Some(track) = state.playlist.get(new_idx) {
            let track = track.clone();
            drop(state);
            let _ = self.play(&track);
        }
    }

    pub fn next(&self) {
        let state = self.state.lock().unwrap();
        if state.playlist.is_empty() {
            return;
        }
        let current_idx = state
            .playlist
            .iter()
            .position(|t| state.current_track.as_ref().map(|c| c.id == t.id).unwrap_or(false))
            .unwrap_or(0);

        let new_idx = match state.play_mode {
            PlayMode::RepeatOne => current_idx,
            PlayMode::Shuffle => {
                if state.playlist.len() > 1 {
                    let mut rng = rand::thread_rng();
                    let mut idx = rand::Rng::gen_range(&mut rng, 0..state.playlist.len());
                    if idx == current_idx {
                        idx = (idx + 1) % state.playlist.len();
                    }
                    idx
                } else {
                    0
                }
            }
            PlayMode::RepeatList => (current_idx + 1) % state.playlist.len(),
        };

        if let Some(track) = state.playlist.get(new_idx) {
            let track = track.clone();
            drop(state);
            let _ = self.play(&track);
        }
    }

    pub fn seek(&self, position_ms: u64) {
        self.send(PlayerCommand::Seek(position_ms));
    }

    pub fn set_volume(&self, volume: f32) {
        self.send(PlayerCommand::SetVolume(volume));
    }

    pub fn set_mode(&self, mode: PlayMode) {
        self.send(PlayerCommand::SetMode(mode));
    }

    pub fn get_state(&self) -> PlayerState {
        let state = self.state.lock().unwrap();
        PlayerState {
            state: state.play_state.clone(),
            mode: state.play_mode.clone(),
            current_track_id: state.current_track.as_ref().map(|t| t.id),
            position: state.position,
            duration: state.duration,
            volume: state.volume,
        }
    }
}

fn audio_thread_main(
    receiver: Receiver<PlayerCommand>,
    state: Arc<Mutex<PlayerStateInternal>>,
    play_ack: Arc<PlayAck>,
) {
    let (stream, stream_handle) = match OutputStream::try_default() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create audio output stream: {}", e);
            return;
        }
    };

    let sink = match Sink::try_new(&stream_handle) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create audio sink: {}", e);
            return;
        }
    };

    let mut decoder: Option<AudioDecoder> = None;
    let mut play_start_time: Option<Instant> = None;
    let mut paused_duration = Duration::ZERO;
    let mut seek_target: Option<u64> = None;
    let mut seek_offset: f64 = 0.0;

    loop {
        let mut should_decode = false;

        while let Ok(cmd) = receiver.try_recv() {
            match cmd {
                PlayerCommand::Play(track) => {
                    sink.stop();
                    sink.play();
                    paused_duration = Duration::ZERO;
                    seek_target = None;
                    seek_offset = 0.0;

                    {
                        let mut s = state.lock().unwrap();
                        s.current_track = Some(track.clone());
                        s.position = 0.0;
                        s.duration = track.duration as f64;
                        s.play_state = PlayState::Playing;
                        s.total_decoded_frames = 0;
                        s.last_position_update = Instant::now();
                    }

                    match AudioDecoder::new(std::path::Path::new(&track.path)) {
                        Ok(d) => {
                            let sample_rate = d.sample_rate();
                            let channels = d.channels();
                            decoder = Some(d);
                            play_start_time = Some(Instant::now());
                            sink.set_volume(state.lock().unwrap().volume);
                            should_decode = true;
                            log::info!("Playing: {} ({}Hz, {}ch)", track.title.as_deref().unwrap_or("Unknown"), sample_rate, channels);

                            {
                                let mut ack = play_ack.0.lock().unwrap();
                                *ack = Some(Ok(()));
                                play_ack.1.notify_one();
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create decoder for {}: {}", track.path, e);
                            {
                                let mut s = state.lock().unwrap();
                                s.play_state = PlayState::Stopped;
                            }

                            {
                                let mut ack = play_ack.0.lock().unwrap();
                                *ack = Some(Err(format!("无法播放文件: {}", track.title.as_deref().unwrap_or("未知"))));
                                play_ack.1.notify_one();
                            }

                            try_auto_advance(&state, &receiver, &sink, &mut decoder, &mut play_start_time, &mut paused_duration, &mut seek_target, &mut seek_offset);
                        }
                    }
                }
                PlayerCommand::Pause => {
                    sink.pause();
                    if let Some(start) = play_start_time {
                        paused_duration += start.elapsed();
                    }
                    play_start_time = None;
                    state.lock().unwrap().play_state = PlayState::Paused;
                }
                PlayerCommand::Resume => {
                    let saved_pos = state.lock().unwrap().position;
                    sink.play();
                    play_start_time = Some(Instant::now());
                    paused_duration = Duration::ZERO;
                    seek_offset = saved_pos;
                    state.lock().unwrap().play_state = PlayState::Playing;
                    should_decode = true;
                }
                PlayerCommand::Stop => {
                    sink.stop();
                    decoder = None;
                    play_start_time = None;
                    paused_duration = Duration::ZERO;
                    seek_target = None;
                    seek_offset = 0.0;
                    let mut s = state.lock().unwrap();
                    s.play_state = PlayState::Stopped;
                    s.position = 0.0;
                    s.total_decoded_frames = 0;
                    s.last_position_update = Instant::now();
                }
                PlayerCommand::Seek(pos_ms) => {
                    sink.stop();
                    sink.play();
                    seek_target = Some(pos_ms);
                    if let Some(d) = decoder.as_mut() {
                        if let Err(e) = d.seek(pos_ms) {
                            log::error!("Seek failed: {}", e);
                        }
                    }
                    paused_duration = Duration::ZERO;
                    play_start_time = Some(Instant::now());
                    seek_offset = pos_ms as f64 / 1000.0;
                    let mut s = state.lock().unwrap();
                    s.position = seek_offset;
                    s.total_decoded_frames = 0;
                    s.last_position_update = Instant::now();
                }
                PlayerCommand::SetVolume(vol) => {
                    sink.set_volume(vol);
                    state.lock().unwrap().volume = vol;
                }
                PlayerCommand::SetMode(mode) => {
                    state.lock().unwrap().play_mode = mode;
                }
                PlayerCommand::SetPlaylist(tracks) => {
                    state.lock().unwrap().playlist = tracks;
                }
            }
        }

        if seek_target.is_some() {
            play_start_time = Some(Instant::now());
            paused_duration = Duration::ZERO;
            seek_target = None;
        }

        {
            let s = state.lock().unwrap();
            if s.play_state == PlayState::Playing {
                should_decode = true;
            }
        }

        if should_decode {
            if let Some(d) = decoder.as_mut() {
                let channels = d.channels();
                match d.decode() {
                    Ok(buf) => {
                        let sample_rate = buf.spec().rate();
                        let num_frames = buf.frames();
                        let samples = convert_audio_buffer(buf);

                        let source = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);
                        sink.append(source);

                        let elapsed_secs = if let Some(start) = play_start_time {
                            (start.elapsed() - paused_duration).as_secs_f64().max(0.0) + seek_offset
                        } else {
                            0.0
                        };

                        let mut s = state.lock().unwrap();
                        s.total_decoded_frames += num_frames as u64;
                        let now = Instant::now();
                        if now.duration_since(s.last_position_update) >= Duration::from_millis(100) {
                            s.position = elapsed_secs;
                            s.last_position_update = now;
                        }
                    }
                    Err(symphonia::core::errors::Error::IoError(_)) => {
                        decoder = None;
                    }
                    Err(e) => {
                        log::error!("Decode error: {}", e);
                        let mut s = state.lock().unwrap();
                        s.play_state = PlayState::Stopped;
                        drop(s);
                        decoder = None;
                        play_start_time = None;
                        paused_duration = Duration::ZERO;
                        try_auto_advance(&state, &receiver, &sink, &mut decoder, &mut play_start_time, &mut paused_duration, &mut seek_target, &mut seek_offset);
                    }
                }
            }
        }

        {
            let s = state.lock().unwrap();
            if s.play_state == PlayState::Playing {
                drop(s);
                let elapsed_secs = if let Some(start) = play_start_time {
                    (start.elapsed() - paused_duration).as_secs_f64().max(0.0) + seek_offset
                } else {
                    0.0
                };
                let now = Instant::now();
                let mut s = state.lock().unwrap();
                if now.duration_since(s.last_position_update) >= Duration::from_millis(100) {
                    s.position = elapsed_secs;
                    s.last_position_update = now;
                }
            }
        }

        {
            let next_track: Option<Track> = {
                let s = state.lock().unwrap();
                if s.play_state == PlayState::Playing && s.duration > 0.0 && s.position >= s.duration - 0.5 {
                    let next_idx = match &s.current_track {
                        Some(current) => {
                            s.playlist.iter().position(|t| t.id == current.id).map(|p| p + 1).unwrap_or(0)
                        }
                        None => 0,
                    };
                    let next_idx = if next_idx >= s.playlist.len() { 0 } else { next_idx };
                    s.playlist.get(next_idx).cloned()
                } else {
                    None
                }
            };
            if let Some(track) = next_track {
                sink.stop();
                sink.play();
                decoder = None;
                play_start_time = None;
                paused_duration = Duration::ZERO;
                seek_target = None;
                seek_offset = 0.0;
                let mut s = state.lock().unwrap();
                s.current_track = Some(track.clone());
                s.position = 0.0;
                s.duration = track.duration as f64;
                s.play_state = PlayState::Playing;
                s.total_decoded_frames = 0;
                s.last_position_update = Instant::now();
                drop(s);
                match AudioDecoder::new(std::path::Path::new(&track.path)) {
                    Ok(d) => {
                        decoder = Some(d);
                        play_start_time = Some(Instant::now());
                    }
                    Err(e) => {
                        log::error!("Failed to load next track: {}", e);
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

fn try_auto_advance(
    state: &Arc<Mutex<PlayerStateInternal>>,
    receiver: &Receiver<PlayerCommand>,
    sink: &Sink,
    decoder: &mut Option<AudioDecoder>,
    play_start_time: &mut Option<Instant>,
    paused_duration: &mut Duration,
    seek_target: &mut Option<u64>,
    seek_offset: &mut f64,
) {
    while let Ok(cmd) = receiver.try_recv() {
        match cmd {
            PlayerCommand::Play(track) => {
                *decoder = None;
                *play_start_time = None;
                *paused_duration = Duration::ZERO;
                *seek_target = None;
                *seek_offset = 0.0;

                {
                    let mut s = state.lock().unwrap();
                    s.current_track = Some(track.clone());
                    s.position = 0.0;
                    s.duration = track.duration as f64;
                    s.play_state = PlayState::Playing;
                    s.total_decoded_frames = 0;
                    s.last_position_update = Instant::now();
                }

                match AudioDecoder::new(std::path::Path::new(&track.path)) {
                    Ok(d) => {
                        *decoder = Some(d);
                        sink.stop();
                        sink.play();
                        *play_start_time = Some(Instant::now());
                        return;
                    }
                    Err(e) => {
                        log::error!("Failed to create decoder: {}", e);
                        state.lock().unwrap().play_state = PlayState::Stopped;
                    }
                }
            }
            _ => {}
        }
    }

    let s = state.lock().unwrap();
    if s.playlist.is_empty() {
        return;
    }
    let current_idx = s
        .playlist
        .iter()
        .position(|t| s.current_track.as_ref().map(|c| c.id == t.id).unwrap_or(false))
        .unwrap_or(0);

    let next_idx = match s.play_mode {
        PlayMode::RepeatOne => current_idx,
        PlayMode::Shuffle => {
            if s.playlist.len() > 1 {
                let mut rng = rand::thread_rng();
                let mut idx = rand::Rng::gen_range(&mut rng, 0..s.playlist.len());
                if idx == current_idx {
                    idx = (idx + 1) % s.playlist.len();
                }
                idx
            } else {
                0
            }
        }
        PlayMode::RepeatList => (current_idx + 1) % s.playlist.len(),
    };

        if let Some(track) = s.playlist.get(next_idx) {
        let track = track.clone();
        drop(s);
        {
            let mut s = state.lock().unwrap();
            s.current_track = Some(track.clone());
            s.position = 0.0;
            s.duration = track.duration as f64;
            s.play_state = PlayState::Playing;
            s.total_decoded_frames = 0;
            s.last_position_update = Instant::now();
        }
        match AudioDecoder::new(std::path::Path::new(&track.path)) {
            Ok(d) => {
                *decoder = Some(d);
                sink.stop();
                sink.play();
                *play_start_time = Some(Instant::now());
                *paused_duration = Duration::ZERO;
                *seek_target = None;
                *seek_offset = 0.0;
            }
            Err(e) => {
                log::error!("Failed to create decoder for next track: {}", e);
                state.lock().unwrap().play_state = PlayState::Stopped;
            }
        }
    }
}

lazy_static::lazy_static! {
    pub static ref PLAYER: PlayerHandle = PlayerHandle::new();
}
