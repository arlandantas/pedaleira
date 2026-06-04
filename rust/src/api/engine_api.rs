use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::engine::handle::EngineHandle;
use crate::engine::runtime::{Runtime, RuntimeConfig};

static ENGINE: Mutex<Option<(Runtime, EngineHandle)>> = Mutex::new(None);
static MUTE: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

/// Start the audio engine. Call stop_engine first if already running.
#[flutter_rust_bridge::frb(sync)]
pub fn start_engine(
    wav_path: String,
    play_output: bool,
    write_output: bool,
    output_path: String,
) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("engine already running; call stop_engine first".to_string());
    }
    let muted = Arc::new(AtomicBool::new(false));
    {
        let mut m = MUTE.lock().map_err(|e| e.to_string())?;
        *m = Some(muted.clone());
    }
    let config = RuntimeConfig { wav_path, play_output, write_output, output_path };
    let (runtime, handle) = Runtime::start(config, muted)?;
    *guard = Some((runtime, handle));
    Ok(())
}

/// Stop the audio engine and release all resources.
#[flutter_rust_bridge::frb(sync)]
pub fn stop_engine() {
    if let Ok(mut guard) = ENGINE.lock() {
        *guard = None;
    }
    if let Ok(mut m) = MUTE.lock() {
        *m = None;
    }
}

/// Toggle bypass for a slot (0=noise gate … 9=boost).
#[flutter_rust_bridge::frb(sync)]
pub fn toggle_bypass(slot: u8, bypass: bool) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some((_, handle)) => handle.toggle_bypass(slot, bypass),
        None => Err("engine not running".to_string()),
    }
}

/// Set params for a slot via JSON string.
#[flutter_rust_bridge::frb(sync)]
pub fn set_param(slot: u8, json: String) -> Result<(), String> {
    let mut guard = ENGINE.lock().map_err(|e| e.to_string())?;
    match guard.as_mut() {
        Some((_, handle)) => handle.set_param(slot, &json),
        None => Err("engine not running".to_string()),
    }
}

/// Mute or unmute the audio output. Persists until changed; not part of preset state.
#[flutter_rust_bridge::frb(sync)]
pub fn set_mute(muted: bool) -> Result<(), String> {
    let guard = MUTE.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(flag) => {
            flag.store(muted, Ordering::Relaxed);
            Ok(())
        }
        None => Err("engine not running".to_string()),
    }
}
