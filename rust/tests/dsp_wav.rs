use hound::{WavReader, WavWriter, WavSpec, SampleFormat};

pub fn load_wav_mono_f32(path: &str) -> Vec<f32> {
    let mut reader = WavReader::open(path).expect("open wav");
    let spec = reader.spec();
    match spec.sample_format {
        SampleFormat::Float => {
            reader.samples::<f32>().map(|s| s.unwrap()).collect()
        }
        SampleFormat::Int => {
            let max = (1u32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.unwrap() as f32 / max).collect()
        }
    }
}

pub fn save_wav_mono_f32(path: &str, samples: &[f32], sample_rate: u32) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).expect("create wav");
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        writer.write_sample((clamped * 32767.0) as i16).unwrap();
    }
}

pub fn ensure_test_di_wav(path: &str) {
    use std::f32::consts::TAU;
    if std::path::Path::new(path).exists() { return; }
    std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for i in 0u32..(44100 * 3) {
        // Guitar-like: low E string fundamental + harmonics
        let t = i as f32 / 44100.0;
        let s = (TAU * 82.4 * t).sin() * 0.5
              + (TAU * 82.4 * 2.0 * t).sin() * 0.25
              + (TAU * 82.4 * 3.0 * t).sin() * 0.15;
        writer.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16).unwrap();
    }
}
