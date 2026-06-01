# Real-Time Guitar Effects App - Project Specification (MVP)

## 1. Project Overview
A real-time, low-latency audio effects engine and control application designed for guitarists. The project uses a hybrid architecture, combining a high-performance DSP engine with a cross-platform mobile UI.

* **UI/Frontend:** Flutter (Dart)
* **Audio Engine/Backend:** Rust
* **Bridge:** `flutter_rust_bridge`
* **Audio API:** `cpal` (for OS-level low-latency audio negotiation)

## 2. Core Architecture

### Production ("Golden") Architecture
* **Hardware (The Brain):** A physical pedalboard containing a Raspberry Pi running a real-time Linux kernel (e.g., Patchbox OS). It handles 100% of the audio DSP. It includes an I2C OLED screen (for preset names) and physical GPIO footswitches.
* **UI (The Remote):** A smartphone running the Flutter app. It connects wirelessly to the Pi (Bluetooth LE/WiFi) to send lightweight control messages (JSON/Protobuf) to adjust parameters. The phone does *not* process audio.

### Development Architecture (Current Phase)
* **Single Device:** Development and testing will be done on a single Kubuntu Linux notebook. Both the Flutter UI and the Rust audio engine will run locally, communicating via `flutter_rust_bridge`.

## 3. Audio Engine Constraints (CRITICAL)
To maintain ultra-low latency (<10-15ms) and prevent audio dropouts (clicking/popping):
* **The Audio Thread is Sacred:** Absolutely no memory allocation (`Vec::new()`, `String`), garbage collection, console printing, or blocking mutexes inside the audio processing loop.
* **State Syncing:** Communication between the Flutter UI thread and the Rust audio thread must use lock-free data structures (e.g., atomic variables, bounded SPSC queues like `crossbeam` or `ringbuf`).

## 4. The MVP Effects Chain (Fixed Routing)
To optimize CPU usage and simplify UI state management, the MVP uses a fixed 8-slot serial chain, plus global Reverb. 

**Chain Order:**
1.  **Noise Gate:** Kills input hum and high-gain hiss.
2.  **Compressor:** Dynamic control and sustain.
3.  **Transparent Overdrive:** Light crunch / boost.
4.  **Distortion:** Heavy rock/metal rhythm gain.
5.  **Fuzz:** Vintage, sputtering, nonlinear clipping.
6.  **Chorus:** 80s/Modern spatial texture.
7.  **Tremolo:** Rhythmic volume modulation (vintage blues/surf).
8.  **Delay:** Echo and depth.
* *Global Output:* **Reverb** (Room/Amp simulation applied at the end of the chain).

## 5. UI/UX Design (Flutter)
Designed to prevent the "fat finger" problem on mobile screens and emulate professional modelers (like Neural DSP / Helix).

* **Main View:** A 2x4 grid displaying the 8 fixed pedals.
* **Live/Stomp Mode:** A single tap on a pedal toggles it ON or OFF (Bypass).
* **Edit Mode:** A long-press (or tapping a small gear icon) opens a full-screen view of that specific pedal to adjust its knobs (Gain, Tone, Level, etc.).
* **Presets:** The top/bottom of the screen displays the current Preset Name with `< >` arrows. Changing a preset instantly loads the saved ON/OFF states and knob parameters for all 8 pedals simultaneously, eliminating the "tap dance."

## 6. Development & Testing Strategy (Kubuntu)

Testing DSP algorithms with a live guitar plugged into a laptop 3.5mm jack causes impedance mismatch (muddy tone) and OS latency. We will use DI (Direct Injection) `.wav` tracks for development.

### Phase 1: Algorithm Development (Pure Rust)
Use this method for writing and perfecting the DSP math (Distortion, Chorus, etc.).
* **Tool:** The `hound` Rust crate.
* **Workflow:** Load a raw guitar DI `.wav` file into memory (array of floats). Run the effect function over the array instantly. Save the output to a new `.wav` file and listen.
* **Why:** It is deterministic, immune to buffer crashes, and easy to step-debug.

### Phase 2: UI Integration & Live Testing (Rust + Flutter)
Use this method to test the `flutter_rust_bridge` connection, ensuring UI knobs successfully update the DSP engine without locking threads.
* **Tools:** `cpal` (Rust) and `pavucontrol` (Kubuntu PulseAudio/PipeWire).
* **Workflow:** 1. Create a Linux virtual audio cable (`module-null-sink`).
    2. Play the DI `.wav` track in a media player (like VLC) on a loop.
    3. Use `pavucontrol` to route VLC's output into the virtual sink.
    4. Set the Rust `cpal` app's input to listen to the "Monitor" of that virtual sink.
* **Why:** Simulates a live, real-time input stream cleanly without requiring an external USB audio interface during early software development.
