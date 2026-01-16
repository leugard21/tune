use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    _stream: OutputStream,
    sink: Sink,
    pub state: PlaybackState,
    pub current_track: Option<String>,
    elapsed: Arc<Mutex<Duration>>,
    pub volume: f32,
    pub muted: bool,
    pub pre_mute_volume: f32,
}

impl Player {
    pub fn new() -> Result<Self, String> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to open audio: {}", e))?;

        let sink = Sink::connect_new(stream.mixer());

        Ok(Self {
            _stream: stream,
            sink,
            state: PlaybackState::Stopped,
            current_track: None,
            elapsed: Arc::new(Mutex::new(Duration::ZERO)),
            volume: 1.0,
            muted: false,
            pre_mute_volume: 1.0,
        })
    }

    pub fn play(&mut self, path: &std::path::Path, track_name: &str) -> Result<(), String> {
        self.stop();

        let file_bytes = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
        let byte_len = file_bytes.len() as u64;
        let cursor = std::io::Cursor::new(file_bytes);

        let source = Decoder::builder()
            .with_data(cursor)
            .with_seekable(true)
            .with_byte_len(byte_len)
            .build()
            .map_err(|e| format!("Failed to decode: {}", e))?;

        *self.elapsed.lock().unwrap() = Duration::ZERO;

        self.sink.append(source);
        self.sink.set_volume(self.volume);
        self.sink.play();

        self.state = PlaybackState::Playing;
        self.current_track = Some(track_name.to_string());

        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => {
                self.sink.pause();
                self.state = PlaybackState::Paused;
            }
            PlaybackState::Paused => {
                self.sink.play();
                self.state = PlaybackState::Playing;
            }
            PlaybackState::Stopped => {}
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.state = PlaybackState::Stopped;
        self.current_track = None;
        *self.elapsed.lock().unwrap() = Duration::ZERO;

        self.sink = Sink::connect_new(self._stream.mixer());
        self.sink.set_volume(self.volume);
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty() && self.state == PlaybackState::Playing
    }

    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn set_volume(&mut self, volume: f32) {
        let rounded_volume = (volume * 10.0).round() / 10.0;
        self.volume = rounded_volume.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    pub fn increase_volume(&mut self) {
        self.set_volume(self.volume + 0.1);
    }

    pub fn decrease_volume(&mut self) {
        self.set_volume(self.volume - 0.1);
    }

    pub fn toggle_mute(&mut self) {
        if self.muted {
            self.set_volume(self.pre_mute_volume);
            self.muted = false;
        } else {
            self.pre_mute_volume = self.volume;
            self.set_volume(0.0);
            self.muted = true;
        }
    }

    pub fn seek(&mut self, duration: Duration) {
        self.sink.try_seek(duration).ok();
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio player")
    }
}
