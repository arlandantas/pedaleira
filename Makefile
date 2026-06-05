CARGO   = $(HOME)/.cargo/bin/cargo
FLUTTER = $(HOME)/flutter/bin/flutter
ANDROID_DEVICE = $(shell adb devices | awk 'NR==2{print $$1}')

.PHONY: test build check render run run-android flutter-build build-android build-android-release codegen clean

## Rust DSP (Phase 1)
test:
	cd rust && $(CARGO) test

check:
	cd rust && $(CARGO) check

build:
	cd rust && $(CARGO) build --release

render:
	cd rust && $(CARGO) test render_all -- --nocapture

## Flutter app (Phase 2+)
run:
	$(FLUTTER) run -d linux

run-android:
	$(FLUTTER) run -d $(ANDROID_DEVICE)

flutter-build:
	$(FLUTTER) build linux

build-android:
	$(FLUTTER) build apk --debug

build-android-release:
	$(FLUTTER) build apk --release

codegen:
	flutter_rust_bridge_codegen generate

clean:
	rm -rf build/
	cd rust && $(CARGO) clean
