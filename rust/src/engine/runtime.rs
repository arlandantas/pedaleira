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
    pub fn start(
        config: RuntimeConfig,
        muted: Arc<AtomicBool>,
    ) -> Result<(Runtime, EngineHandle), String> {
        let (wav_samples, sample_rate) = load_wav(&config.wav_path)?;

        let (engine, cmd_prod) = make_engine(sample_rate as f32);
        let handle = EngineHandle::new(cmd_prod);

        let (mut sample_prod, mut sample_cons) = HeapRb::<f32>::new(4096).split();

        let (mut sink_prod_opt, sink_cons_opt) = if config.write_output {
            let (p, c) = HeapRb::<f32>::new(4096).split();
            (Some(p), Some(c))
        } else {
            (None, None)
        };

        let shutdown = Arc::new(AtomicBool::new(false));

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

        let host = cpal::default_host();

        // TODO(Phase 4): expose device list via bridge API so the Flutter UI
        // can let the user pick the output device at runtime.
        //
        // Build candidate list: system default first, then all enumerated devices.
        // Filter by capability before building the stream to avoid "device unavailable"
        // errors from hw: or virtual devices that reject f32/mono streams.
        let mut candidates: Vec<cpal::Device> = Vec::new();
        if let Some(d) = host.default_output_device() { candidates.push(d); }
        if let Ok(devs) = host.output_devices() {
            for d in devs {
                let name = d.name().unwrap_or_default();
                if !candidates.iter().any(|c| c.name().unwrap_or_default() == name) {
                    candidates.push(d);
                }
            }
        }
        let device = candidates.into_iter()
            .find(|d| {
                d.supported_output_configs()
                    .map(|configs| configs
                        .filter(|c| c.channels() == 1)
                        .any(|c| c.min_sample_rate().0 <= sample_rate
                                && sample_rate <= c.max_sample_rate().0))
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!(
                "no output device supports mono {}Hz; check PulseAudio/PipeWire is running",
                sample_rate
            ))?;

        eprintln!("[engine] using output device: {}", device.name().unwrap_or_else(|_| "<unknown>".into()));

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
                if !play_output || muted.load(Ordering::Relaxed) {
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
