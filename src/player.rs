use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    pub state: PlaybackState,
    pub current_track: Option<String>,
    elapsed: Arc<Mutex<Duration>>,
}

impl Player {
    pub fn new() -> Result<Self, String> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|e| format!("Failed to open audio: {}", e))?;

        let sink =
            Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create sink: {}", e))?;

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            state: PlaybackState::Stopped,
            current_track: None,
            elapsed: Arc::new(Mutex::new(Duration::ZERO)),
        })
    }

    pub fn play(&mut self, path: &Path, track_name: &str) -> Result<(), String> {
        self.stop();

        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let source =
            Decoder::new(BufReader::new(file)).map_err(|e| format!("Failed to decode: {}", e))?;

        *self.elapsed.lock().unwrap() = Duration::ZERO;

        self.sink.append(source);
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

        if let Ok(sink) = Sink::try_new(&self._stream_handle) {
            self.sink = sink;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty() && self.state == PlaybackState::Playing
    }

    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio player")
    }
}
