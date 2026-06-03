use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::JoinHandle;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleRate, BufferSize};
use ringbuf::{traits::*, HeapRb};

use crate::engine::make_engine;
use crate::engine::handle::EngineHandle;

// Safety: cpal's ALSA Stream is Send-safe on Linux/ALSA. Gate to Linux to
// prevent silent UB if compiled for macOS (CoreAudio) or Windows (WASAPI).
#[cfg(target_os = "linux")]
unsafe impl Send for Runtime {}

pub struct RuntimeConfig {
    pub wav_path: String,
    pub play_output: bool,
    pub write_output: bool,
    pub output_path: String,
}

pub struct Runtime {
    _stream: Stream,
    reader_handle: Option<JoinHandle<()>>,
    sink_handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = StreamTrait::pause(&self._stream);
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(h) = self.reader_handle.take() { let _ = h.join(); }
        if let Some(h) = self.sink_handle.take() { let _ = h.join(); }
    }
}

impl Runtime {
    pub fn start(config: RuntimeConfig) -> Result<(Runtime, EngineHandle), String> {
        // 1. Load WAV into memory
        let (wav_samples, sample_rate) = load_wav(&config.wav_path)?;

        // 2. Engine + command channel
        let (engine, cmd_prod) = make_engine(sample_rate as f32);
        let handle = EngineHandle::new(cmd_prod);

        // 3. Sample ring: reader thread → audio callback
        // 4096 samples ≈ 93 ms at 44100 Hz — comfortably larger than typical ALSA buffer (256–2048 samples)
        let (mut sample_prod, mut sample_cons) = HeapRb::<f32>::new(4096).split();

        // 4. File sink ring: audio callback → writer thread (optional)
        let (mut sink_prod_opt, sink_cons_opt) = if config.write_output {
            let (p, c) = HeapRb::<f32>::new(4096).split();
            (Some(p), Some(c))
        } else {
            (None, None)
        };

        let shutdown = Arc::new(AtomicBool::new(false));

        // 5. WAV reader thread: loops wav_samples → sample ring
        let shutdown_reader = shutdown.clone();
        let reader_handle = std::thread::spawn(move || {
            let len = wav_samples.len();
            let mut idx = 0usize;
            loop {
                if shutdown_reader.load(Ordering::Relaxed) { break; }
                let s = wav_samples[idx];
                idx = (idx + 1) % len;
                loop {
                    if shutdown_reader.load(Ordering::Relaxed) { return; }
                    if sample_prod.try_push(s).is_ok() { break; }
                    std::thread::sleep(std::time::Duration::from_micros(100));
                }
            }
        });

        // 6. File sink thread (optional)
        let sink_handle = if config.write_output {
            let mut cons = sink_cons_opt.unwrap();
            let shutdown_sink = shutdown.clone();
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };
            let mut writer = hound::WavWriter::create(&config.output_path, spec)
                .map_err(|e| e.to_string())?;
            Some(std::thread::spawn(move || {
                loop {
                    let done = shutdown_sink.load(Ordering::Relaxed);
                    while let Some(s) = cons.try_pop() {
                        writer.write_sample(s).ok();
                    }
                    if done && cons.is_empty() { break; }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                writer.finalize().ok();
            }))
        } else {
            None
        };

        // 7. cpal output stream
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| "no default output device".to_string())?;
        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Default,
        };
        let play_output = config.play_output;
        let mut engine = engine;

        let stream = device.build_output_stream(
            &stream_config,
            move |data: &mut [f32], _| {
                for s in data.iter_mut() {
                    *s = sample_cons.try_pop().unwrap_or(0.0);
                }
                engine.process_block(data);
                if let Some(ref mut sp) = sink_prod_opt {
                    for &s in data.iter() { let _ = sp.try_push(s); }
                }
                if !play_output {
                    for s in data.iter_mut() { *s = 0.0; }
                }
            },
            |err| eprintln!("cpal stream error: {err}"),
            None,
        ).map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        Ok((
            Runtime { _stream: stream, reader_handle: Some(reader_handle), sink_handle, shutdown },
            handle,
        ))
    }
}

/// Load a mono WAV file (any bit depth, float or int) into a Vec<f32>.
fn load_wav(path: &str) -> Result<(Vec<f32>, u32), String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err(format!("only mono WAV supported (got {} channels); convert with: ffmpeg -i input.mp3 -ac 1 -ar 44100 -sample_fmt s16 output.wav", spec.channels));
    }
    let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| e.to_string()))
            .collect::<Result<Vec<f32>, String>>()?,
        hound::SampleFormat::Int => reader
            .samples::<i32>()
            .map(|s| s.map(|v| v as f32 / scale).map_err(|e| e.to_string()))
            .collect::<Result<Vec<f32>, String>>()?,
    };
    Ok((samples, spec.sample_rate))
}
