use std::{
    fs::File,
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    PlaybackState,
    spectrum::{SpectrumChunk, TapSource, default_spectrum, try_start_spectrum_worker},
};
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};

enum Command {
    Load(PathBuf, bool),
    Stop,
    Toggle,
    SetPaused(bool, mpsc::SyncSender<bool>),
    Seek(f32),
    Volume(f32),
}

struct Backend {
    tx: mpsc::Sender<Command>,
    state: Arc<Mutex<PlaybackState>>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_enabled: Arc<AtomicBool>,
}

static BACKEND: OnceLock<Backend> = OnceLock::new();

fn backend() -> &'static Backend {
    BACKEND.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(PlaybackState {
            position: 0.0,
            duration: 0.0,
            paused: true,
            volume: 1.0,
            error: String::new(),
            spectrum: default_spectrum(),
        }));
        let spectrum = Arc::new(Mutex::new(default_spectrum()));
        let spectrum_enabled = Arc::new(AtomicBool::new(true));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (fft_tx, fft_rx) = mpsc::sync_channel::<SpectrumChunk>(4);
            try_start_spectrum_worker(fft_rx, Arc::clone(&spectrum), Arc::clone(&spectrum_enabled));
            let thread_state = Arc::clone(&state);
            let thread_spectrum_enabled = Arc::clone(&spectrum_enabled);
            thread::Builder::new()
                .name("zeedle-audio".into())
                .spawn(move || {
                    log::info!("audio backend thread running");
                    let mut sink = DeviceSinkBuilder::open_default_sink();
                    if let Ok(sink) = &mut sink {
                        sink.log_on_drop(false);
                    }
                    let player = sink
                        .as_ref()
                        .ok()
                        .map(|sink| Player::connect_new(sink.mixer()));
                    if let Err(error) = &sink {
                        log::error!("audio output unavailable: {error}");
                        thread_state.lock().unwrap().error =
                            format!("Audio output unavailable: {error}");
                    }
                    let mut active_duration = 0.0;
                    let mut last_update = Instant::now();

                    loop {
                        match rx.recv_timeout(Duration::from_millis(60)) {
                            Ok(Command::Load(path, autoplay)) => {
                                log::info!("loading audio file: {}", path.display());
                                let result = File::open(&path)
                                    .map_err(|error| error.to_string())
                                    .and_then(|file| {
                                        Decoder::try_from(file).map_err(|error| error.to_string())
                                    });
                                match (player.as_ref(), result) {
                                    (Some(player), Ok(source)) => {
                                        active_duration = source
                                            .total_duration()
                                            .map_or(0.0, |d| d.as_secs_f32());
                                        player.clear();
                                        player.pause();
                                        player.append(TapSource::new(
                                            source,
                                            fft_tx.clone(),
                                            Arc::clone(&thread_spectrum_enabled),
                                        ));
                                        player.set_volume(thread_state.lock().unwrap().volume);
                                        if autoplay {
                                            player.play();
                                        }
                                        let mut state = thread_state.lock().unwrap();
                                        state.position = 0.0;
                                        state.duration = active_duration;
                                        state.paused = !autoplay;
                                        state.error.clear();
                                    }
                                    (_, Err(error)) => {
                                        log::error!("failed to load {}: {error}", path.display());
                                        thread_state.lock().unwrap().error = error;
                                    }
                                    (None, Ok(_)) => {}
                                }
                            }
                            Ok(Command::SetPaused(paused, reply)) => {
                                let actual = if let Some(player) = player.as_ref() {
                                    if paused {
                                        player.pause();
                                    } else if !player.empty() {
                                        player.play();
                                    }
                                    player.is_paused()
                                } else {
                                    true
                                };
                                thread_state.lock().unwrap().paused = actual;
                                let _ = reply.send(actual);
                            }
                            Ok(Command::Toggle) => {
                                if let Some(player) = player.as_ref() {
                                    if !player.empty() {
                                        if player.is_paused() {
                                            player.play();
                                        } else {
                                            player.pause();
                                        }
                                    }
                                }
                            }
                            Ok(Command::Stop) => {
                                if let Some(player) = player.as_ref() {
                                    player.clear();
                                }
                                active_duration = 0.0;
                                let mut state = thread_state.lock().unwrap();
                                state.position = 0.0;
                                state.duration = 0.0;
                                state.paused = true;
                            }
                            Ok(Command::Seek(seconds)) => {
                                if let Some(player) = player.as_ref() {
                                    if !player.empty() {
                                        if let Err(error) = player
                                            .try_seek(Duration::from_secs_f32(seconds.max(0.0)))
                                        {
                                            log::error!("failed to seek to {seconds}s: {error}");
                                            thread_state.lock().unwrap().error = error.to_string();
                                        }
                                    }
                                }
                            }
                            Ok(Command::Volume(volume)) => {
                                if let Some(player) = player.as_ref() {
                                    player.set_volume(volume);
                                }
                                thread_state.lock().unwrap().volume = volume;
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                        }

                        if last_update.elapsed() >= Duration::from_millis(80) {
                            if let Some(player) = player.as_ref() {
                                let mut state = thread_state.lock().unwrap();
                                if !player.empty() {
                                    state.position = player.get_pos().as_secs_f32();
                                    state.paused = player.is_paused();
                                } else if active_duration > 0.0 {
                                    state.position = active_duration;
                                    state.paused = true;
                                }
                            }
                            last_update = Instant::now();
                        }
                    }
                })
                .expect("could not start audio worker");
        }
        #[cfg(target_arch = "wasm32")]
        drop(rx);
        Backend {
            tx,
            state,
            spectrum,
            spectrum_enabled,
        }
    })
}

pub(crate) async fn set_paused(paused: bool) -> bool {
    let (reply, received) = mpsc::sync_channel(1);
    if backend()
        .tx
        .send(Command::SetPaused(paused, reply))
        .is_err()
    {
        return true;
    }
    received.recv().unwrap_or(true)
}

pub(crate) fn play(path: String, autoplay: bool) {
    let _ = backend()
        .tx
        .send(Command::Load(PathBuf::from(path), autoplay));
}

pub(crate) fn stop() {
    let _ = backend().tx.send(Command::Stop);
}
pub(crate) fn toggle() {
    let _ = backend().tx.send(Command::Toggle);
}
pub(crate) fn seek(seconds: f32) {
    let _ = backend().tx.send(Command::Seek(seconds));
}
pub(crate) fn set_volume(volume: f32) {
    let _ = backend().tx.send(Command::Volume(volume.clamp(0.0, 1.0)));
}

pub(crate) fn set_spectrum_enabled(enabled: bool) {
    let backend = backend();
    backend.spectrum_enabled.store(enabled, Ordering::Relaxed);
    if !enabled {
        *backend.spectrum.lock().unwrap() = default_spectrum();
    }
}

pub(crate) fn state() -> String {
    let backend = backend();
    let mut state = backend.state.lock().unwrap().clone();
    state.spectrum = backend.spectrum.lock().unwrap().clone();
    serde_json::to_string(&state).unwrap_or_else(|_| "{}".into())
}
