use rust_lib_pedaleira::dsp::boost::Boost;

#[test]
fn boost_unity_passes_signal_unchanged() {
    let mut b = Boost::new(1.0, 44100.0);
    let mut buf = vec![0.5f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
}

#[test]
fn boost_gain_2_doubles_signal() {
    let mut b = Boost::new(2.0, 44100.0);
    let mut buf = vec![0.25f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| (s - 0.5).abs() < 1e-6));
}

#[test]
fn boost_gain_0_silences_signal() {
    let mut b = Boost::new(0.0, 44100.0);
    let mut buf = vec![0.5f32; 64];
    b.process(&mut buf);
    assert!(buf.iter().all(|&s| s == 0.0));
}

#[test]
fn boost_set_gain_updates_multiplier() {
    let mut b = Boost::new(1.0, 44100.0);
    b.set_gain(3.0);
    // Process enough samples for the smoother to converge (~50ms at 44100Hz).
    let mut buf = vec![0.1f32; 4096];
    b.process(&mut buf);
    // Only check the tail where the smoother has settled.
    assert!(buf[3000..].iter().all(|&s| (s - 0.3).abs() < 0.01),
        "gain should converge to 3.0; last sample = {}", buf[4095]);
}
